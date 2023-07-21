use crate::ui::menu::MenuState;

/// Сохраняет состояние вкладки графика
#[derive(Debug)]
pub struct GraphState {
    /// Содержит возможные значения датчиков
    pub sensor_fields: Vec<String>,

    /// Содержит возможные серийники датчиков
    pub serials: Option<Vec<String>>,

    /// Содержит параметры для X
    pub x_states: [MenuState; 4],

    /// Содержит параметры для всех Y
    pub ys_states: Vec<[MenuState; 4]>,

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
            x_states: Default::default(),
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
    pub fn selected_menu_state(&self) -> Option<&MenuState> {
        self.selected.and_then(|i| match i {
            0..=3 => self.x_states.get(i),
            _ => self.ys_states[i / 4 - 1].get(i % 4),
        })
    }

    /// Возвращает изменяемую ссылку на состояние выделенного элемента меню
    pub fn selected_menu_state_mut(&mut self) -> Option<&mut MenuState> {
        self.selected.and_then(|i| match i {
            0..=3 => self.x_states.get_mut(i),
            _ => self.ys_states[i / 4 - 1].get_mut(i % 4),
        })
    }

    /// Выбирает предыдущий элемент
    pub fn select_prev(&mut self, step: usize) {
        if let Some(i) = self.selected.as_mut() {
            let length = (self.ys_states.len() + 1) * 4;
            *i = (*i + length - step) % length;
        }
    }

    /// Выбирает следующий элемент
    pub fn select_next(&mut self, step: usize) {
        if let Some(i) = self.selected.as_mut() {
            let length = (self.ys_states.len() + 1) * 4;
            *i = (*i + step) % length;
        }
    }
}
