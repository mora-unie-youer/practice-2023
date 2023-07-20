use tui_tree_widget::{TreeItem, TreeState};

use crate::filepicker::state::FilePickerState;

/// Сохраняет состояние вкладки дерева датчиков
#[derive(Debug)]
pub struct SensorsState<'a> {
    /// Сохраняет состояние дерева датчиков
    pub tree_state: TreeState,
    /// Сохраняет элементы дерева датчиков
    pub items: Vec<TreeItem<'a>>,

    /// Сохраняет состояние элемента выбора файлов
    pub file_picker_state: Option<FilePickerState>,
}

impl Default for SensorsState<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorsState<'_> {
    /// Создаёт новый экземпляр состояния вкладки дерева датчиков
    pub fn new() -> Self {
        Self {
            tree_state: TreeState::default(),
            items: Vec::new(),

            file_picker_state: None,
        }
    }
}
