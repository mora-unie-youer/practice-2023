use std::collections::HashMap;

use crate::app::state::App;

use super::SensorsFields;

impl App<'_> {
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
