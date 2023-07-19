use tui_tree_widget::{TreeItem, TreeState};

/// Определяет дерево датчиков
#[derive(Default, Debug)]
pub struct SensorsTree<'a> {
    pub state: TreeState,
    pub items: Vec<TreeItem<'a>>,
}

impl<'a> SensorsTree<'a> {
    /// Отправляет модулю Tree нажатие кнопки "вниз"
    pub fn down(&mut self) {
        self.state.key_down(&self.items);
    }

    /// Отправляет модулю Tree нажатие кнопки "вверх"
    pub fn up(&mut self) {
        self.state.key_up(&self.items);
    }

    /// Отправляет модулю Tree нажатие кнопки "влево"
    pub fn left(&mut self) {
        self.state.key_left();
    }

    /// Отправляет модулю Tree нажатие кнопки "вправо"
    pub fn right(&mut self) {
        self.state.key_right();
    }
}
