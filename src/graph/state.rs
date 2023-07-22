use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    database::{SensorsFields, SensorsSerials},
    ui::{input::InputState, menu::MenuState},
};

/// Сохраняет состояние вкладки графика
#[derive(Debug)]
pub struct GraphState {
    /// Сохраняет все поля датчиков
    pub sensor_fields: Rc<RefCell<SensorsFields>>,

    /// Сохраняет все серийники датчиков
    pub sensor_serials: Rc<RefCell<SensorsSerials>>,

    /// Сохраняет все поля данных X
    pub x_data_fields: Vec<String>,

    /// Сохраняет все поля данных Y
    pub y_data_fields: Vec<String>,

    /// Содержит параметры для X
    pub x_states: [GraphFieldState; 4],

    /// Содержит параметры для всех Y
    pub ys_states: Vec<[GraphFieldState; 4]>,

    /// Содержит индекс выделенного виджета
    pub selected: Option<usize>,
}

impl GraphState {
    /// Создаёт новый экземпляр состояния вкладки графика
    pub fn new(
        sensor_fields: Rc<RefCell<SensorsFields>>,
        sensor_serials: Rc<RefCell<SensorsSerials>>,
    ) -> Self {
        GraphState {
            sensor_fields,
            sensor_serials,

            x_states: GraphState::default_graph(),
            ys_states: vec![GraphState::default_graph()],

            x_data_fields: vec![],
            y_data_fields: vec![],

            selected: None,
        }
    }

    /// Обновляет всё, связанное с данными датчиков в графике
    pub fn update_sensor_data(&mut self) {
        self.update_sensor_fields();
    }

    /// Обновляет поля датчиков и связанное с ними в графике
    pub fn update_sensor_fields(&mut self) {
        // Поля, которые необходимо игнорировать
        const IGNORE_FIELDS: [&str; 3] = ["id", "serial", "date"];

        // Получаем поля сенсоров для дерева
        let sensor_fields_ref = self.sensor_fields.borrow();
        let mut sensor_fields: Vec<_> = sensor_fields_ref.iter().collect();
        sensor_fields.sort_unstable();

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
            let y_data_field = y_fields[0].menu_mut().unwrap();
            // Берём сохранённый индекс в поле
            if let Some(i) = y_data_field.selected() {
                y_data_field.set_select(index_conversion_map.get(&i).cloned());
            }
        }

        // Теперь можно сохранить поля данных Y
        self.y_data_fields = new_y_data_fields;
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
