use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
};

use itertools::Itertools;

/// HashMap, хранящий все поля отдельных датчиков
/// Используется для того, чтобы можно было удобно импортировать данные в БД
/// Представляет из себя зависимость "название датчика -> поля"
type SensorsFields = HashMap<String, HashSet<String>>;

/// Нормализация имени сенсора, чтобы сделать его пригодным для SQLite
fn normalize_sensor_name(name: String) -> String {
    name.replace(|ch: char| ch == '-' || ch.is_whitespace(), "_")
}

/// Из набора данных о датчиках получает поля каждого датчика
fn get_all_sensors_fields(data: &serde_json::Value) -> SensorsFields {
    let mut sensors_fields = HashMap::new();

    // Итерируем по каждому вхождению о каком-то датчике
    for (_, entry) in data.as_object().unwrap() {
        // Получаем название датчика
        let uname = entry["uName"].as_str().unwrap().to_owned();
        // Нормализуем название датчика для последующего использования (меняем пробелы и - на _)
        let uname = normalize_sensor_name(uname);

        // Получаем поля, связанные с этим датчиком
        let fields: HashSet<String> = entry["data"].as_object().unwrap().keys().cloned().collect();

        // Получаем уже сохранённые поля этого датчика, либо вставляем эти
        let saved_fields = sensors_fields
            .entry(uname.clone())
            .or_insert(fields.clone());

        // Получаем пересечение полей, чтобы найти универсальный перечень полей
        // (Спасибо Паскаль-11, за то что ты так уникальный)
        let mut intersection: HashSet<String> =
            saved_fields.intersection(&fields).cloned().collect();

        // TODO: make some UI to choose it on import stage, and filter out here
        intersection.retain(|field| !field.starts_with("system_"));
        intersection.retain(|field| !field.ends_with("_date"));
        intersection.retain(|field| !field.ends_with("_time"));

        // Записываем новые поля, если нужно
        if saved_fields.len() != intersection.len() {
            sensors_fields
                .entry(uname)
                .and_modify(|fields| *fields = intersection);
        }
    }

    // Удаляем поля сенсоров, которые в итоге вышли пустыми
    sensors_fields.retain(|_, v| !v.is_empty());
    sensors_fields
}

// Возвращает SQL запрос на создание таблицы для датчика
fn create_table_sql_query(sensor: &str, fields: &[String]) -> String {
    // Получаем поля для таблицы и добавляем туда номер прибора и дату
    let mut fields: Vec<_> = fields.iter().map(|field| format!("{field} REAL")).collect();
    fields.push("serial TEXT".to_owned());
    fields.push("date TEXT".to_owned());

    // Подготавливаем SQL запрос на создание БД
    let fields = fields.join(",").replace('-', "_");
    format!("CREATE TABLE IF NOT EXISTS {sensor} (id INTEGER PRIMARY KEY, {fields}, UNIQUE(serial, date))")
}

// Возвращает SQL запрос на добавление данных в таблицу датчика
fn insert_entry_sql_query(sensor: &str, fields: &[String]) -> String {
    let fields_names = fields.join(",").replace('-', "_");
    let fields_places = (1..=fields.len() + 2).map(|i| format!("?{i}")).join(",");
    format!("INSERT INTO {sensor} ({fields_names},serial,date) VALUES ({fields_places})")
}

/// Из набора данных и набора полей датчиков сделать импорт данных в БД
fn import_data_to_database(
    database: &mut rusqlite::Connection,
    data: &serde_json::Value,
    sensors_fields: &SensorsFields,
) -> Result<(), Box<dyn std::error::Error>> {
    // Делаем HashMap массивов, чтобы меньше клонировать пришлось
    let sensors_fields: HashMap<String, Vec<String>> = sensors_fields
        .iter()
        .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
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
        let uname = entry["uName"].as_str().unwrap().to_owned();
        let uname = normalize_sensor_name(uname);

        // Если такого датчика мы не храним, то просто пропускаем
        if !sensors_fields.contains_key(&uname) {
            continue;
        }

        // Получаем номер датчика и дату
        let serial = entry["serial"].as_str().unwrap().to_owned();
        let date = entry["Date"].as_str().unwrap().to_owned();

        // Получаем данные датчика
        let data = &entry["data"].as_object().unwrap();
        let mut fetched_fields: Vec<_> = sensors_fields[&uname]
            .iter()
            .map(|field| data[field].as_str().unwrap().to_owned())
            .collect();
        fetched_fields.push(serial);
        fetched_fields.push(date);

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut database = rusqlite::Connection::open("db.sqlite")?;

    // Загружаем 28 дней данных
    for i in 3..=30 {
        let file = File::open(format!("data/2023-03-{i:0>2}.json"))?;
        let reader = BufReader::new(file);
        let json: serde_json::Value = serde_json::from_reader(reader)?;

        let sensors_fields = get_all_sensors_fields(&json);
        import_data_to_database(&mut database, &json, &sensors_fields)?;
    }

    Ok(())
}
