use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use chrono::NaiveDateTime;
use itertools::Itertools;

use crate::{
    database::{SensorsFields, SensorsSerials},
    ui::{input::InputState, menu::MenuState},
};

/// Сохраняет состояние вкладки графика
#[derive(Debug)]
pub struct GraphState {
    /// Позволяет получить доступ к базе данных
    database: Arc<Mutex<rusqlite::Connection>>,

    /// Сохраняет все поля датчиков
    pub sensor_fields: Rc<RefCell<SensorsFields>>,

    /// Сохраняет все серийники датчиков
    pub sensor_serials: Rc<RefCell<SensorsSerials>>,

    /// Сохраняет все поля данных X
    pub x_data_fields: Vec<String>,

    /// Сохраняет все поля данных Y
    pub y_data_fields: Vec<String>,

    /// Содержит в себе поля Y без дополнительными вариантами
    pub y_data_fields_without_extra: SensorsFields,

    /// Сохраняет возможыне выборы серийника
    pub serial_fields: SensorsSerials,

    /// Содержит параметры для X
    pub x_states: [GraphFieldState; 4],

    /// Содержит параметры для всех Y
    pub ys_states: Vec<[GraphFieldState; 4]>,

    /// Содержит все данные графиков для построения
    pub datasets: Vec<Vec<Vec<(f64, f64)>>>,

    /// Содержит границы данных ((x_min, x_max), (y_min, y_max))
    pub dataset_ranges: ((f64, f64), (f64, f64)),

    /// Содержит флаг того, что поля обновлялись
    pub was_edited: bool,

    /// Содержит индекс выделенного виджета
    pub selected: Option<usize>,
}

impl GraphState {
    /// Создаёт новый экземпляр состояния вкладки графика
    pub fn new(
        database: Arc<Mutex<rusqlite::Connection>>,
        sensor_fields: Rc<RefCell<SensorsFields>>,
        sensor_serials: Rc<RefCell<SensorsSerials>>,
    ) -> Self {
        GraphState {
            database,
            sensor_fields,
            sensor_serials,

            x_states: GraphState::default_graph(),
            ys_states: vec![GraphState::default_graph()],

            x_data_fields: vec![],
            y_data_fields: vec![],
            y_data_fields_without_extra: HashMap::new(),
            serial_fields: SensorsSerials::new(),

            datasets: vec![vec![]],
            dataset_ranges: Default::default(),
            was_edited: false,

            selected: None,
        }
    }

    /// Обновляет всё, связанное с данными датчиков в графике
    pub fn update_sensor_data(&mut self) {
        self.update_sensor_fields();
        self.update_sensor_serials();
    }

    /// Обновляет поля датчиков и связанное с ними в графике
    pub fn update_sensor_fields(&mut self) {
        // Поля, которые необходимо игнорировать
        const IGNORE_FIELDS: [&str; 3] = ["id", "serial", "date"];

        // Получаем поля сенсоров
        let sensor_fields_ref = self.sensor_fields.borrow();
        let mut sensor_fields: Vec<_> = sensor_fields_ref.iter().collect();
        sensor_fields.sort_unstable();

        // Генерируем хэш-таблицу с полями каждого датчика для Y
        let new_y_data_fields_without_extra: SensorsFields = sensor_fields_ref
            .iter()
            .map(|(sensor, fields)| {
                let new_fields = fields
                    .iter()
                    .filter(|field| !IGNORE_FIELDS.contains(&field.as_str()))
                    .map(|field| format!("{sensor}/{field}"))
                    .collect();
                (sensor.clone(), new_fields)
            })
            .collect();
        self.y_data_fields_without_extra = new_y_data_fields_without_extra;

        // Генерируем поля данных для X
        let mut new_x_data_fields = vec!["date".to_owned()];
        new_x_data_fields.extend(sensor_fields.iter().flat_map(|(sensor, fields)| {
            fields
                .iter()
                .filter(|field| !IGNORE_FIELDS.contains(&field.as_str()))
                .map(move |field| format!("{sensor}/{field}"))
        }));

        // Преобразуем старый индекс в поле X, если он был
        let x_data_field = self.x_states[0].menu_mut().unwrap();
        if let Some(i) = x_data_field.selected() {
            // Получаем данное значение поля данных X
            let value = &self.x_data_fields[i];
            // Ищем новый индекс и записываем его
            x_data_field.set_select(new_x_data_fields.iter().position(|field| field == value));
        }

        // Теперь можно сохранить поля данных X
        self.x_data_fields = new_x_data_fields;

        // Дропаем ссылку, т.к. она мешает дальнейшему коду
        drop(sensor_fields_ref);

        // Обновляем поля данных Y
        self.update_y_data_fields();
    }

    pub fn update_y_data_fields(&mut self) {
        const EXTRA_FIELDS: [&str; 1] = ["Эфф. темп."];

        // Генерируем поля данных для Y на основе полей X
        let mut new_y_data_fields = self.x_data_fields.clone();
        new_y_data_fields.retain(|field| field.contains('/'));

        // Если X специализирует единственный датчик - убираем все другие датчики
        if let Some(i) = self.x_states[0].menu().unwrap().selected() {
            // Получаем данное значение поля данных X
            let value = &self.x_data_fields[i];
            // Если поле представляет из себя поле датчика, то убираем лишние из Y
            if let Some((sensor, _)) = value.split_once('/') {
                let start = format!("{sensor}/");
                new_y_data_fields.retain(|field| field.starts_with(&start));
            }
        }

        // Генерируем поля с дополнительными вариантами
        let mut new_y_data_fields = new_y_data_fields.clone();
        // Дополняем поля к последнему датчику
        let last_field = new_y_data_fields.last().cloned().unwrap();
        let (last_sensor, _) = last_field.split_once('/').unwrap();
        for extra_field in EXTRA_FIELDS {
            new_y_data_fields.push(format!("{last_sensor}/{extra_field}"));
        }

        // Дополняем новые поля к каждому датчику
        for (i, window) in new_y_data_fields.clone().windows(2).enumerate().rev() {
            let (sensor_a, _) = window[0].split_once('/').unwrap();
            let (sensor_b, _) = window[1].split_once('/').unwrap();
            // Когда два соседних сенсора не равны - это место для вставки дополнительных полей
            if sensor_a != sensor_b {
                for extra_field in EXTRA_FIELDS {
                    new_y_data_fields.insert(i + 1, format!("{sensor_a}/{extra_field}"));
                }
            }
        }

        // Конвертируем все выбранные Y поля на новые индексы
        let index_conversion_map: HashMap<usize, usize> = new_y_data_fields
            .iter()
            .enumerate()
            .filter_map(|(i, field)| {
                self.y_data_fields
                    .iter()
                    .position(|old| old == field)
                    .map(|j| (j, i))
            })
            .collect();

        // Редактируем все установленные поля данных Y
        for y_fields in &mut self.ys_states {
            // Конверируем поля данных Y
            let field = y_fields[0].menu_mut().unwrap();
            // Берём сохранённый индекс в поле
            if let Some(i) = field.selected() {
                field.set_select(index_conversion_map.get(&i).copied());
            }
        }

        // Теперь можно сохранить поля данных Y
        self.y_data_fields = new_y_data_fields;
    }

    /// Обновляет серийники датчиков и связанное с ними в графике
    pub fn update_sensor_serials(&mut self) {
        // TODO: в будущем обдумать это. Вероятнее всего просто убрать
        // const EXTRA_FIELDS: [&str; 3] = ["Средн.", "Макс.", "Мин."];

        // Получаем серийники датчиков
        let sensor_serials_ref = self.sensor_serials.borrow();
        // Делаем копию, чтобы добавить новые "псевдо-серийники"
        let serial_fields = sensor_serials_ref.clone();

        // Добавляем дополнительные поля в начало
        // for (_, fields) in serial_fields.iter_mut() {
        //     for extra_field in EXTRA_FIELDS.into_iter().rev() {
        //         fields.insert(0, extra_field.to_owned());
        //     }
        // }

        // Теперь можно сохранить серийники датчиков
        self.serial_fields = serial_fields;
    }

    /// Обновляет датасеты графиков, если это необходимо
    pub fn update_datasets(&mut self) {
        // Если ничего не редактировалось - не обновляем
        if !self.was_edited {
            return;
        }

        // Получаем какой X мы хотим. Если он не установлен, выходим из функции обновления
        let x_data_index = match self.x_states[0].menu().unwrap().selected() {
            Some(i) => i,
            None => return,
        };
        let x_data = &self.x_data_fields[x_data_index];

        // Получаем сенсор, поле и серийник, если есть
        let (_x_sensor, x_field, x_serial) = {
            // Если поле X состоит из {sensor}/{field}, разбираем его
            if let Some((sensor, field)) = x_data.split_once('/') {
                // Получаем серийник датчика. Если он не установлен, выходим из обновления
                let serial_index = match self.x_states[1].menu().unwrap().selected() {
                    Some(i) => i,
                    None => return,
                };
                let serial = self.serial_fields[sensor][serial_index].as_str();
                (Some(sensor), field, Some(serial))
            } else {
                (None, x_data.as_str(), None)
            }
        };

        // Получаем диапазон X
        let x_min = self.x_states[2].input().unwrap().content();
        let x_max = self.x_states[3].input().unwrap().content();

        // Конвертируем эти диапазоны в разные типы
        let x_min_date = match NaiveDateTime::parse_from_str(x_min, "%Y-%m-%d %H:%M:%S") {
            Ok(date) => Some(date),
            Err(_) if x_field == "date" && !x_min.is_empty() => return,
            _ => None,
        };

        let x_min_float = match x_min.parse::<f64>() {
            Ok(float) => Some(float),
            Err(_) if x_field != "date" && !x_min.is_empty() => return,
            _ => None,
        };

        let x_max_date = match NaiveDateTime::parse_from_str(x_max, "%Y-%m-%d %H:%M:%S") {
            Ok(date) => Some(date),
            Err(_) if x_field == "date" && !x_max.is_empty() => return,
            _ => None,
        };

        let x_max_float = match x_max.parse::<f64>() {
            Ok(float) => Some(float),
            Err(_) if x_field != "date" && !x_max.is_empty() => return,
            _ => None,
        };

        // Составляем заранее часть SQL запроса
        let x_filtering = match (x_field == "date", !x_min.is_empty(), !x_max.is_empty()) {
            (true, true, true) => format!(
                "date BETWEEN {} AND {}",
                x_min_date.unwrap().timestamp(),
                x_max_date.unwrap().timestamp()
            ),
            (true, _, true) => format!("date < {}", x_max_date.unwrap().timestamp()),
            (true, true, _) => format!("date > {}", x_min_date.unwrap().timestamp()),

            (_, true, true) => format!(
                "{x_field} BETWEEN {} AND {}",
                x_min_float.unwrap(),
                x_max_float.unwrap()
            ),
            (_, _, true) => format!("{x_field} < {}", x_max_float.unwrap()),
            (_, true, _) => format!("{x_field} > {}", x_min_float.unwrap()),
            _ => String::new(),
        };
        let x_ordering = format!("ORDER BY {x_field}");

        // Открываем соединение с базой данных
        let database = self.database.lock().unwrap();

        // Обрабатываем Y данные
        let (mut y_all_min, mut y_all_max) = (std::f64::MAX, std::f64::MIN);
        let new_datasets = self
            .ys_states
            .iter()
            .map(|y_states| {
                let (data, y_min, y_max) = self.generate_datasets_for_y_states(
                    &database,
                    x_field,
                    x_serial,
                    &x_filtering,
                    &x_ordering,
                    y_states,
                );

                // Устанавливаем новые минимум и максимум для графика
                y_all_min = y_all_min.min(y_min);
                y_all_max = y_all_max.max(y_max);

                data
            })
            .collect();
        self.datasets = new_datasets;

        // Записываем диапазоны значений
        let x_min = match (x_min_date, x_min_float) {
            (Some(date), _) => date.timestamp() as f64,
            (_, Some(float)) => float,
            _ => self
                .datasets
                .iter()
                .flatten()
                .map(|dataset| {
                    dataset
                        .iter()
                        .map(|&(x, _)| x)
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap()
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap(),
        };

        let x_max = match (x_max_date, x_max_float) {
            (Some(date), _) => date.timestamp() as f64,
            (_, Some(float)) => float,
            _ => self
                .datasets
                .iter()
                .flatten()
                .map(|dataset| {
                    dataset
                        .iter()
                        .map(|&(x, _)| x)
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap()
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap(),
        };
        self.dataset_ranges = ((x_min, x_max), (y_all_min, y_all_max));

        // Ставим флаг того, что изменений нет
        self.was_edited = false;
    }

    /// Генерирует датасеты графиков и получает в них минимальное и максимальное значение
    fn generate_datasets_for_y_states(
        &self,
        database: &rusqlite::Connection,
        x_field: &str,
        x_serial: Option<&str>,

        x_filtering: &str,
        x_ordering: &str,

        y_states: &[GraphFieldState; 4],
    ) -> (Vec<Vec<(f64, f64)>>, f64, f64) {
        // Получаем какой Y мы хотим. Если он не установлен, пропускаем эту функцию
        let y_data_index = match y_states[0].menu().unwrap().selected() {
            Some(i) => i,
            None => return (vec![vec![]], 0., 0.),
        };
        let y_data = &self.y_data_fields[y_data_index];

        // Получаем сенсор, поле и серийник, если есть
        let (y_sensor, y_field, y_serial) = {
            // Поле Y гарантированно состоит из {sensor}/{field}, разбираем его
            let (sensor, field) = y_data.split_once('/').unwrap();

            // Получаем серийник датчика. Если он не установлен, пропускаем
            let serial = if let Some(serial) = x_serial {
                // Если установлен серийник у X, используем его
                serial
            } else {
                // Иначе пытаемся получить серийник у Y
                let serial_index = match y_states[1].menu().unwrap().selected() {
                    Some(i) => i,
                    None => return (vec![vec![]], 0., 0.),
                };
                &self.serial_fields[sensor][serial_index]
            };

            (sensor, field, serial)
        };

        // Проверяем дополнительные датчики, если они есть
        let y_field_extra1 = if let Some(menu_state) = y_states[2].menu() {
            // Пытаемся получить первое дополнительное поле у Y. Если не удаётся, пропускаем
            let extra_index = match menu_state.selected() {
                Some(i) => i,
                None => return (vec![vec![]], 0., 0.),
            };
            let (_, extra) = self.y_data_fields_without_extra[y_sensor][extra_index]
                .split_once('/')
                .unwrap();
            Some(extra)
        } else {
            None
        };

        let y_field_extra2 = if let Some(menu_state) = y_states[3].menu() {
            // Пытаемся получить первое дополнительное поле у Y. Если не удаётся, пропускаем
            let extra_index = match menu_state.selected() {
                Some(i) => i,
                None => return (vec![vec![]], 0., 0.),
            };
            let (_, extra) = self.y_data_fields_without_extra[y_sensor][extra_index]
                .split_once('/')
                .unwrap();
            Some(extra)
        } else {
            None
        };

        // Составляем часть SQL запроса для отфильтровки серийников
        let serial_filtering = match y_serial {
            "Средн." | "Мин." | "Макс." => String::new(),
            serial => format!("serial = '{serial}'"),
        };

        // Составляем часть SQL запроса со всеми фильтрами
        let sql_filtering = match (!x_filtering.is_empty(), !serial_filtering.is_empty()) {
            (true, true) => format!("WHERE {x_filtering} AND {serial_filtering}"),
            (false, false) => String::new(),
            _ => format!("WHERE {x_filtering}{serial_filtering}"),
        };

        // Поля, которые нам необходимо собрать
        let graph_fields = match (y_field_extra1, y_field_extra2) {
            (Some(field1), Some(field2)) => format!("{field1},{field2}"),
            _ => y_field.to_owned(),
        };
        let select_fields = format!("{graph_fields},{x_field},serial");

        // Собираем финальный SQL запрос
        let sql = format!("SELECT {select_fields} FROM {y_sensor} {sql_filtering} {x_ordering}");
        let mut statement = database.prepare(&sql).unwrap();

        // Выполняем запрос SQL и сохраняем данные
        let mut rows = statement.query(()).unwrap();
        let mut data = vec![];
        let (mut y_min, mut y_max) = (std::f64::MAX, std::f64::MIN);

        // Читаем строки из БД
        if y_field_extra2.is_some() {
            let conversion = match y_field {
                "Эфф. темп." => |y1: f64, y2: f64| {
                    let t = y1;
                    let h = y2;
                    t - 0.4 * (t - 10.) * (1. - h / 100.)
                },
                _ => panic!("Неизвестно, что делать с данными"),
            };

            // Если у нас были дополнительные поля, высчитываем сразу значение по формуле
            while let Some(row) = rows.next().unwrap() {
                // Получаем данные со строки
                let y1: f64 = row.get(0).unwrap();
                let y2: f64 = row.get(1).unwrap();
                let x: f64 = row.get(2).unwrap();

                // Обрабатываем дополнительные поля и кладём в массив
                let y = conversion(y1, y2);
                data.push((x, y));
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        } else {
            // Просто кладём полученные данные в массив
            while let Some(row) = rows.next().unwrap() {
                let y: f64 = row.get(0).unwrap();
                let x: f64 = row.get(1).unwrap();
                data.push((x, y));
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        }

        // Проверяем, нужно ли нам график разделять на несколько
        let dataset = if x_field == "date" {
            // Если X - дата, то не нужно
            vec![data]
        } else {
            // Если X - какое-то поле датчика - нужно
            let (mut min, mut avg, mut max) = (vec![], vec![], vec![]);
            for (x, points) in &data.into_iter().group_by(|&(x, _)| x) {
                let ys: Vec<_> = points.map(|(_, y)| y).collect();
                let ys_len = ys.len();

                let (mut y_min, mut y_max, mut y_sum) = (std::f64::MAX, std::f64::MIN, 0.);
                for y in ys {
                    y_min = y_min.min(y);
                    y_max = y_max.max(y);
                    y_sum += y;
                }

                min.push((x, y_min));
                max.push((x, y_max));
                avg.push((x, y_sum / ys_len as f64));
            }

            // Возвращаем три разделённых графика
            vec![min, avg, max]
        };

        (dataset, y_min, y_max)
    }

    /// Возвращает серийники для поля с индексом
    pub fn get_serial_fields_for_sensor(&self, field_index: usize) -> &Vec<String> {
        // Извлекаем название сенсора из выбранного поля данных
        let selection = if field_index == 1 {
            // Обрабатываем как поле X
            let selection = self.x_states[0].menu().unwrap().selected().unwrap();
            &self.x_data_fields[selection]
        } else {
            // Обрабатываем как поле Y
            let selection = self.ys_states[field_index / 4 - 1][0]
                .menu()
                .unwrap()
                .selected()
                .unwrap();
            &self.y_data_fields[selection]
        };

        let (sensor, _) = selection.split_once('/').unwrap();
        // Получаем серийники датчика
        &self.serial_fields[sensor]
    }

    /// Возвращает дефолтные поля пустого графика
    pub fn default_graph() -> [GraphFieldState; 4] {
        [
            GraphFieldState::new_menu(),
            GraphFieldState::Hidden,
            GraphFieldState::Hidden,
            GraphFieldState::Hidden,
        ]
    }

    /// Возвращает ссылку на состояние выделенного элемента меню
    pub fn selected_field_state(&self) -> &GraphFieldState {
        match self.selected.unwrap() {
            i @ 0..=3 => &self.x_states[i],
            i => &self.ys_states[i / 4 - 1][i % 4],
        }
    }

    /// Возвращает изменяемую ссылку на состояние выделенного элемента меню
    pub fn selected_field_state_mut(&mut self) -> &mut GraphFieldState {
        match self.selected.unwrap() {
            i @ 0..=3 => &mut self.x_states[i],
            i => &mut self.ys_states[i / 4 - 1][i % 4],
        }
    }

    /// Возвращает ссылку на состояние выделенное поле ввода текста
    pub fn selected_input_state(&self) -> &InputState {
        self.selected_field_state().input().unwrap()
    }

    /// Возвращает изменяемую ссылку на состояние выделенное поле ввода текста
    pub fn selected_input_state_mut(&mut self) -> &mut InputState {
        self.selected_field_state_mut().input_mut().unwrap()
    }

    /// Возвращает ссылку на состояние выделенного элемента меню
    pub fn selected_menu_state(&self) -> &MenuState {
        self.selected_field_state().menu().unwrap()
    }

    /// Возвращает изменяемую ссылку на состояние выделенного элемента меню
    pub fn selected_menu_state_mut(&mut self) -> &mut MenuState {
        self.selected_field_state_mut().menu_mut().unwrap()
    }

    /// Выбирает предыдущий элемент
    pub fn select_prev(&mut self, step: usize) {
        if let Some(i) = self.selected.as_mut() {
            let length = (self.ys_states.len() + 1) * 4;
            *i = (*i + length - step) % length;

            // Если попали на скрытое поле -> надо переходить к следующему
            if let GraphFieldState::Hidden = self.selected_field_state() {
                self.select_prev(1);
            }
        }
    }

    /// Выбирает следующий элемент
    pub fn select_next(&mut self, step: usize) {
        if let Some(i) = self.selected.as_mut() {
            let length = (self.ys_states.len() + 1) * 4;
            *i = (*i + step) % length;

            // Если попали на скрытое поле -> надо переходить к следующему
            if let GraphFieldState::Hidden = self.selected_field_state() {
                self.select_next(1);
            }
        }
    }
}

/// Перечисляемый тип, определяющий вид поля графика
#[derive(Debug)]
pub enum GraphFieldState {
    /// Определяет скрытое поле
    Hidden,

    /// Определяет поле ввода текста
    Input(InputState),

    /// Определяет меню с возможными полями
    Menu(MenuState),
}

impl Default for GraphFieldState {
    fn default() -> Self {
        Self::Hidden
    }
}

impl GraphFieldState {
    /// Создаёт новое состояние поля ввода
    pub fn new_input() -> Self {
        Self::Input(InputState::default())
    }

    /// Создаёт новое состояние меню
    pub fn new_menu() -> Self {
        Self::Menu(MenuState::default())
    }

    /// Возвращает ссылку на состояние поля ввода
    pub fn input(&self) -> Option<&InputState> {
        match self {
            Self::Input(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает изменяемую ссылку на состояние поля ввода
    pub fn input_mut(&mut self) -> Option<&mut InputState> {
        match self {
            Self::Input(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает ссылку на состояние меню
    pub fn menu(&self) -> Option<&MenuState> {
        match self {
            Self::Menu(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает изменяемую ссылку на состояние меню
    pub fn menu_mut(&mut self) -> Option<&mut MenuState> {
        match self {
            Self::Menu(state) => Some(state),
            _ => None,
        }
    }
}
