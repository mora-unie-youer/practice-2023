use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use chrono::NaiveDateTime;
use itertools::Itertools;

use crate::app::App;

/// HashMap, хранящий все поля отдельных датчиков
/// Используется для того, чтобы можно было удобно импортировать данные в БД
/// Представляет из себя зависимость "название датчика -> поля"
type SensorsFields = HashMap<String, Vec<String>>;

/// Нормализация имени сенсора, чтобы сделать его пригодным для SQLite
fn normalize_sensor_name(name: &str) -> String {
    name.replace(|ch: char| ch == '-' || ch.is_whitespace(), "_")
}

/// Из набора данных о датчиках получает поля каждого датчика
fn get_all_sensors_fields(data: &serde_json::Value) -> SensorsFields {
    let mut sensors_fields = HashMap::new();

    // Итерируем по каждому вхождению о каком-то датчике
    for (_, entry) in data.as_object().unwrap() {
        // Получаем название датчика
        let uname = entry["uName"].as_str().unwrap();
        // Нормализуем название датчика для последующего использования (меняем пробелы и - на _)
        let uname = normalize_sensor_name(uname);
        // Получаем поля, связанные с этим датчиком
        let fields: HashSet<&String> = entry["data"].as_object().unwrap().keys().collect();

        // Получаем уже сохранённые поля этого датчика, либо вставляем эти
        let saved_fields = sensors_fields.entry(uname).or_insert({
            let mut new_fields = fields.clone();

            // Отфильтровываем ненужные поля
            // TODO: make some UI to choose it on import stage, and filter out here
            new_fields.retain(|field| {
                !field.starts_with("system_")
                    && !field.starts_with("NTP_")
                    && !field.ends_with("_date")
                    && !field.ends_with("_time")
            });

            new_fields
        });

        // Сохраняем лишь "пересечение" полей
        // (Спасибо Паскаль-11, за то что ты так уникальный)
        saved_fields.retain(|field| fields.contains(field));
    }

    // Удаляем поля сенсоров, которые в итоге вышли пустыми
    sensors_fields
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (k, v.into_iter().cloned().collect()))
        .collect()
}

/// Возвращает SQL запрос на создание таблицы для датчика
fn create_table_sql_query(sensor: &str, fields: &[String]) -> String {
    // Получаем поля для таблицы и добавляем туда номер прибора и дату
    let mut fields: Vec<_> = fields.iter().map(|field| format!("{field} REAL")).collect();
    fields.push("serial TEXT".to_owned());
    fields.push("date INTEGER".to_owned());

    // Подготавливаем SQL запрос на создание БД
    let fields = fields.join(",").replace('-', "_");
    format!("CREATE TABLE IF NOT EXISTS {sensor} (id INTEGER PRIMARY KEY, {fields}, UNIQUE(serial, date))")
}

/// Возвращает SQL запрос на добавление данных в таблицу датчика
fn insert_entry_sql_query(sensor: &str, fields: &[String]) -> String {
    let fields_names = fields.join(",").replace('-', "_");
    let fields_places = (1..=fields.len() + 2).map(|i| format!("?{i}")).join(",");
    format!("INSERT INTO {sensor} ({fields_names},serial,date) VALUES ({fields_places})")
}

impl App<'_> {
    /// Импортирует из набора данных и набора полей датчиков данные в БД
    fn import_data_to_database(
        database: Arc<Mutex<rusqlite::Connection>>,
        data: &serde_json::Value,
        sensors_fields: &SensorsFields,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Получаем соединение с базой данных
        let mut database = database.lock().unwrap();

        // Делаем HashMap массивов, чтобы меньше клонировать пришлось
        let sensors_fields: HashMap<String, Vec<String>> = sensors_fields
            .iter()
            .map(|(k, v)| (k.clone(), v.to_vec()))
            .collect();

        // Здесь будут храниться SQL запросы для таблиц, чтобы не создавать SQL запрос каждый раз
        let mut cached_sql_queries: HashMap<String, String> = HashMap::new();

        // Начинаем транзакцию на создание таблиц
        let create_table_tx = database.transaction()?;
        // Создаём необходимые таблицы
        for (sensor, fields) in &sensors_fields {
            // Подготавливаем SQL запрос для создания строки
            let sql = insert_entry_sql_query(sensor, fields);
            cached_sql_queries.insert(sensor.clone(), sql);

            // Создаём таблицу
            let sql = create_table_sql_query(sensor, fields);
            create_table_tx.execute(&sql, ())?;
        }
        // Заканчиваем транзакцию на создание таблиц
        create_table_tx.commit()?;

        // Начинаем транзакцию на добавление данных
        let insert_entry_tx = database.transaction()?;
        // Итерируем по вхождениям данных
        for (_, entry) in data.as_object().unwrap().into_iter() {
            // Получаем название датчика и нормализуем его
            let uname = entry["uName"].as_str().unwrap();
            let uname = normalize_sensor_name(uname);

            // Если такого датчика мы не храним, то просто пропускаем
            if !sensors_fields.contains_key(&uname) {
                continue;
            }

            // Получаем номер датчика и дату
            let serial = entry["serial"].as_str().unwrap().to_owned();
            let date = entry["Date"].as_str().unwrap().to_owned();
            let date = NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S")?;

            // Получаем данные датчика
            let data = &entry["data"].as_object().unwrap();
            let mut fetched_fields: Vec<_> = sensors_fields[&uname]
                .iter()
                .map(|field| data[field].as_str().unwrap().to_owned())
                .collect();
            fetched_fields.push(serial);
            fetched_fields.push(date.timestamp().to_string());

            // Делаем полученные поля пригодными для библиотеки
            let fetched_fields: Vec<_> = fetched_fields
                .iter()
                .map(|s| s as &dyn rusqlite::ToSql)
                .collect();

            // Выполняем SQL запрос
            let mut statement = insert_entry_tx.prepare_cached(&cached_sql_queries[&uname])?;
            match statement.execute(fetched_fields.as_slice()) {
                Ok(_) => (),
                Err(err) => println!("Не удалось загрузить строку данных по причине: {err}"),
            }
        }
        // Заканчиваем транзакцию на добавление данных
        insert_entry_tx.commit()?;

        Ok(())
    }

    /// Импортирует данные из файла в БД
    pub fn import_file_to_database(&self, file_path: PathBuf) -> JoinHandle<Result<(), ()>> {
        // Делаем "копию" соединения с базой данных
        let database = self.database.clone();

        std::thread::spawn(|| {
            // Открываем файл и читаем JSON
            let file = File::open(file_path).map_err(|_| ())?;
            let reader = BufReader::new(file);
            let json: serde_json::Value = serde_json::from_reader(reader).map_err(|_| ())?;

            // Получаем поля сенсоров из данных и имортируем данные
            let sensors_fields = get_all_sensors_fields(&json);
            Self::import_data_to_database(database, &json, &sensors_fields).map_err(|_| ())?;

            Ok(())
        })
    }

    /// Получает структуру таблиц датчиков, загруженных в БД
    pub fn get_sensors_fields(&self) -> Result<SensorsFields, Box<dyn std::error::Error>> {
        // Делаем "копию" соединения с базой данных
        let database = self.database.clone();
        // Получаем соединение с базой данных
        let database = database.lock().unwrap();

        // Таблица, которая будет хранить все известные поля сенсоров
        let mut sensors_fields = HashMap::new();

        // SQL запрос, который получит структуру таблиц
        let sql = "SELECT name, sql FROM sqlite_schema WHERE type = 'table'";
        let mut statement = database.prepare_cached(sql)?;

        // Получаем строки из БД и итерируем по ним
        let mut rows = statement.query(())?;
        while let Some(row) = rows.next()? {
            // Получаем запрошенные поля
            let name: String = row.get(0)?;
            let sql: String = row.get(1)?;

            // Получаем спецификацию таблицы
            let fields = sql.split_once('(').unwrap().1;
            let fields = fields.rsplit_once(", UNIQUE").unwrap().0;
            let fields: Vec<_> = fields
                .split(',')
                .map(|field| field.trim().split_once(' ').unwrap().0.to_owned())
                .collect();

            // Кладём поля в хэш-таблицу
            sensors_fields.insert(name, fields);
        }

        Ok(sensors_fields)
    }
}
