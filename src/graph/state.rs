use crate::ui::menu::MenuState;

/// Сохраняет состояние вкладки графика
#[derive(Debug)]
pub struct GraphState {
    /// Содержит возможные значения датчиков
    pub sensor_fields: Vec<String>,

    /// Содержит возможные серийники датчиков
    pub serials: Option<Vec<String>>,

    /// Содержит параметры для X
    pub x_states: [GraphFieldState; 4],

    /// Содержит параметры для всех Y
    pub ys_states: Vec<[GraphFieldState; 4]>,

    /// Содержит индекс выделенного виджета
    pub selected: Option<usize>,
}

impl Default for GraphState {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphState {
    /// Создаёт новый экземпляр состояния вкладки графика
    pub fn new() -> Self {
        GraphState {
            x_states: [
                GraphFieldState::Menu(MenuState::default()),
                GraphFieldState::Hidden,
                GraphFieldState::Input(()),
                GraphFieldState::Menu(MenuState::default()),
            ],
            ys_states: vec![Default::default()],

            sensor_fields: vec![
                "Поле 1".to_owned(),
                "Поле 2".to_owned(),
                "Поле 3".to_owned(),
                "Поле 4".to_owned(),
                "Поле 5".to_owned(),
                "Поле 6".to_owned(),
                "Поле 7".to_owned(),
                "Поле 8".to_owned(),
                "Поле 9".to_owned(),
                "Поле 10".to_owned(),
                "Поле 11".to_owned(),
            ],
            serials: None,

            selected: None,
        }
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
    pub fn selected_input_state(&self) -> &() {
        self.selected_field_state().input().unwrap()
    }

    /// Возвращает изменяемую ссылку на состояние выделенное поле ввода текста
    pub fn selected_input_state_mut(&mut self) -> &mut () {
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
    Input(()),

    /// Определяет меню с возможными полями
    Menu(MenuState),
}

impl Default for GraphFieldState {
    fn default() -> Self {
        Self::Input(())
    }
}

impl GraphFieldState {
    /// Возвращает ссылку на состояние поля ввода
    pub fn input(&self) -> Option<&()> {
        match self {
            Self::Input(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает изменяемую ссылку на состояние поля ввода
    pub fn input_mut(&mut self) -> Option<&mut ()> {
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
