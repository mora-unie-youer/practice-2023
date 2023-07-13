use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::BufReader,
};

use chrono::NaiveDateTime;
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

    sensors_fields
}

/// Из набора данных и набора полей датчиков сделать импорт данных в БД
fn import_data_to_database(
    database: &rusqlite::Connection,
    data: &serde_json::Value,
    sensors_fields: &SensorsFields,
) -> Result<(), Box<dyn std::error::Error>> {
    // Создаём необходимые таблицы
    for (sensor, fields) in sensors_fields {
        // Получаем поля для таблицы и добавляем туда номер прибора и дату
        let mut fields: Vec<_> = fields.iter().map(|field| format!("{field} REAL")).collect();
        fields.push("serial TEXT".to_owned());
        fields.push("date TEXT".to_owned());

        // Подготавливаем SQL запрос
        let fields = fields.join(",").replace('-', "_");
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {sensor} (id INTEGER PRIMARY KEY, {fields}, UNIQUE(serial, date))"
        );
        database.execute(&sql, ())?;
    }

    // Сортируем данные по ID (иначе они в беспорядке)
    // Этим же мы гарантируем, что вхождения будут добавлены по дате
    // Думаю это может ускорить БД в обращениях
    // NOTE: похоже это нифига не помогает, данные говно
    let entries: BTreeMap<usize, &serde_json::Value> = data
        .as_object()
        .unwrap()
        .into_iter()
        .map(|(id, value)| (id.parse().unwrap(), value))
        .collect();

    // Начинаем транзакцию
    database.execute("BEGIN", ())?;

    // Итерируем по вхождениям данных
    for (_, entry) in entries.into_iter() {
        // Получаем название датчика и нормализуем его, а также его номер
        let uname = entry["uName"].as_str().unwrap().to_owned();
        let uname = normalize_sensor_name(uname);
        let serial = entry["serial"].as_str().unwrap().to_owned();

        // Получаем дату
        let date = entry["Date"].as_str().unwrap();
        let date = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S")?;

        // Получаем поля, которые нам необходимо импортировать
        let mut fields: Vec<_> = sensors_fields[&uname].iter().cloned().collect();

        // Получаем данные датчика
        let data = &entry["data"].as_object().unwrap();
        let mut fetched_fields: Vec<_> = fields
            .iter()
            .map(|field| data[field].as_str().unwrap().to_owned())
            .collect();

        // Заносим данные в БД
        // Для начала добавим serial и date в поля
        fields.push("serial".to_owned());
        fields.push("date".to_owned());
        fetched_fields.push(serial);
        fetched_fields.push(date.to_string());

        // Подготавливаем SQL запрос и выполняем его
        let fields_places = (1..=fields.len()).map(|i| format!("?{i}")).join(",");
        let fields = fields.join(",").replace('-', "_");
        let sql = format!("INSERT INTO {uname} ({fields}) VALUES ({fields_places})");

        // Делаем полученные поля пригодными для библиотеки
        let fetched_fields: Vec<_> = fetched_fields
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        // Выполняем SQL запрос
        match database.execute(&sql, fetched_fields.as_slice()) {
            Ok(_) => (),
            Err(err) => println!("Не удалось загрузить строку данных по причине: {err}"),
        }
    }

    // Заканчиваем транзакцию
    database.execute("COMMIT", ())?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database = rusqlite::Connection::open("db.sqlite")?;

    // Загружаем 28 дней данных
    for i in 3..=30 {
        let file = File::open(format!("data/2023-03-{i:0>2}.json"))?;
        let reader = BufReader::new(file);
        let json: serde_json::Value = serde_json::from_reader(reader)?;

        let sensors_fields = get_all_sensors_fields(&json);
        import_data_to_database(&database, &json, &sensors_fields)?;
    }

    Ok(())
}
