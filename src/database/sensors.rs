use std::collections::HashMap;

use crate::app::state::App;

use super::{SensorsFields, SensorsSerials};

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

    /// Возвращает серийники датчиков, уже загруженных в состояние приложения
    pub fn get_sensors_serials(&self) -> Result<SensorsSerials, Box<dyn std::error::Error>> {
        // Делаем "копию" соединения с базой данных
        let database = self.database.clone();
        // Получаем соединение с базой данных
        let database = database.lock().unwrap();

        // Таблица, которая будет хранить все известные серийники сенсоров
        let mut sensor_serials = HashMap::new();

        // Получаем с каждого сенсора серийники
        for sensor in self.sensor_fields.borrow().keys() {
            // SQL запрос, который получит серийиники
            let sql = format!("SELECT DISTINCT serial FROM {sensor}");
            let mut statement = database.prepare_cached(&sql)?;
            let mut rows = statement.query(())?;

            // Получаем серийники
            let mut serials: Vec<String> = vec![];
            while let Some(row) = rows.next()? {
                serials.push(row.get(0)?);
            }

            // Кладём серийники в хэш-таблицу
            serials.sort_unstable();
            sensor_serials.insert(sensor.clone(), serials);
        }

        Ok(sensor_serials)
    }
}
