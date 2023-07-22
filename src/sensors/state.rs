use std::{cell::RefCell, rc::Rc};

use tui_tree_widget::{TreeItem, TreeState};

use crate::{database::SensorsFields, filepicker::state::FilePickerState};

/// Сохраняет состояние вкладки дерева датчиков
#[derive(Debug)]
pub struct SensorsState<'a> {
    /// Сохраняет все поля датчиков
    pub sensor_fields: Rc<RefCell<SensorsFields>>,

    /// Сохраняет состояние дерева датчиков
    pub tree_state: TreeState,

    /// Сохраняет элементы дерева датчиков
    pub items: Vec<TreeItem<'a>>,

    /// Сохраняет состояние элемента выбора файлов
    pub file_picker_state: Option<FilePickerState>,
}

impl SensorsState<'_> {
    /// Создаёт новый экземпляр состояния вкладки дерева датчиков
    pub fn new(sensor_fields: Rc<RefCell<SensorsFields>>) -> Self {
        Self {
            sensor_fields,

            tree_state: TreeState::default(),
            items: Vec::new(),

            file_picker_state: None,
        }
    }

    /// Обновляет поля датчиков и связанное с ними в дереве датчиков
    pub fn update_sensor_fields(&mut self) {
        // Получаем поля сенсоров для дерева
        let sensor_fields = self.sensor_fields.borrow();
        let mut sensor_fields: Vec<_> = sensor_fields.iter().collect();
        sensor_fields.sort_unstable();

        // Преобразуем поля в дерево
        let sensors_tree: Vec<_> = sensor_fields
            .into_iter()
            .map(|(name, fields)| {
                TreeItem::new(
                    name.clone(),
                    fields
                        .clone()
                        .into_iter()
                        .map(TreeItem::new_leaf)
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        self.items = sensors_tree;
    }
}
