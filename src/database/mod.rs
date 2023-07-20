use std::collections::HashMap;

pub mod import;
pub mod sensors;

/// HashMap, хранящий все поля отдельных датчиков
/// Используется для того, чтобы можно было удобно импортировать данные в БД
/// Представляет из себя зависимость "название датчика -> поля"
pub type SensorsFields = HashMap<String, Vec<String>>;
