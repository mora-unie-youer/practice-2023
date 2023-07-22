use std::{cell::RefCell, rc::Rc};

use tui_tree_widget::{TreeItem, TreeState};

use crate::{
    database::{SensorsFields, SensorsSerials},
    filepicker::state::FilePickerState,
};

/// Сохраняет состояние вкладки дерева датчиков
#[derive(Debug)]
pub struct SensorsState<'a> {
    /// Сохраняет все поля датчиков
    pub sensor_fields: Rc<RefCell<SensorsFields>>,

    /// Сохраняет все серийники датчиков
    pub sensor_serials: Rc<RefCell<SensorsSerials>>,

    /// Сохраняет состояние дерева датчиков
    pub tree_state: TreeState,

    /// Сохраняет элементы дерева датчиков
    pub items: Vec<TreeItem<'a>>,

    /// Сохраняет состояние элемента выбора файлов
    pub file_picker_state: Option<FilePickerState>,
}

impl SensorsState<'_> {
    /// Создаёт новый экземпляр состояния вкладки дерева датчиков
    pub fn new(
        sensor_fields: Rc<RefCell<SensorsFields>>,
        sensor_serials: Rc<RefCell<SensorsSerials>>,
    ) -> Self {
        Self {
            sensor_fields,
            sensor_serials,

            tree_state: TreeState::default(),
            items: Vec::new(),

            file_picker_state: None,
        }
    }

    /// Обновляет всё, связанное с данными датчиков в графике
    pub fn update_sensor_data(&mut self) {
        // Получаем поля датчиков для дерева
        let sensor_fields = self.sensor_fields.borrow();
        let mut sensor_fields: Vec<_> = sensor_fields.iter().collect();
        sensor_fields.sort_unstable();

        // Получаем серийники датчиков
        let sensor_serials = self.sensor_serials.borrow();
        let mut sensor_serials: Vec<_> = sensor_serials.iter().collect();
        sensor_serials.sort_unstable();

        let sensors_tree = sensor_fields
            .iter()
            .zip(&sensor_serials)
            .map(|((name, fields), (_, serials))| (name, fields, serials))
            .map(|(&name, fields, serials)| {
                let fields: Vec<_> = fields.iter().cloned().map(TreeItem::new_leaf).collect();
                let serials: Vec<_> = serials.iter().cloned().map(TreeItem::new_leaf).collect();

                let fields_tree = TreeItem::new("Поля", fields);
                let serials_tree = TreeItem::new("Серийники", serials);
                TreeItem::new(name.clone(), vec![fields_tree, serials_tree])
            })
            .collect();
        self.items = sensors_tree;
    }
}
