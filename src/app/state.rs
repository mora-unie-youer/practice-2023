use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crossterm::event::{KeyEvent, MouseEvent};

use crate::{
    database::SensorsFields, filepicker::state::FilePickerState, graph::state::GraphState,
    sensors::state::SensorsState,
};

use super::tabs::{TabState, Tabs};

/// Структура, определяющая состояние приложения
/// Используется везде, где только можно
#[derive(Debug)]
pub struct App<'a> {
    /// Определяет, работает ли сейчас программа
    pub running: bool,

    /// Соединение с базой данных
    pub database: Arc<Mutex<rusqlite::Connection>>,

    /// Хранит поля датчиков в данный момент выполнения программы
    pub sensor_fields: Rc<RefCell<SensorsFields>>,

    /// Хранит серийники датчиков в данный момент выполнения программы
    pub sensor_serials: Rc<RefCell<SensorsFields>>,

    /// Определяет вкладки, открытые в приложении
    pub tabs: Tabs<'a>,
}

impl<'a> App<'a> {
    /// Создаёт новое состояние приложения
    pub fn new(database: rusqlite::Connection) -> Result<Self, Box<dyn std::error::Error>> {
        // Делаем базовый экземпляр состояния приложения
        let mut app = Self {
            running: true,
            database: Arc::new(Mutex::new(database)),
            sensor_fields: Rc::new(RefCell::new(SensorsFields::new())),
            sensor_serials: Rc::new(RefCell::new(SensorsFields::new())),
            tabs: Tabs::default(),
        };

        // Подготавливаем первую вкладку - вкладка сенсоров
        let sensors_state =
            SensorsState::new(app.sensor_fields.clone(), app.sensor_serials.clone());
        let app_state = TabState::Sensors(sensors_state);
        app.tabs.open(app_state);

        // Подготавливаем дерево сенсоров
        app.update_sensor_data()?;

        Ok(app)
    }

    /// Выполняет один тик обновления в состоянии приложения
    pub fn tick(&mut self) {
        match self.tabs.state() {
            TabState::Graph(_) => self.tick_graph(),
            TabState::Sensors(_) => self.tick_sensors(),
        }
    }

    /// Обрабатывает все события, связанные с нажатием клавиш
    pub fn on_key_event(&mut self, event: KeyEvent) -> std::io::Result<()> {
        match self.tabs.state() {
            TabState::Graph(_) => self.on_key_event_graph(event),
            TabState::Sensors(_) => self.on_key_event_sensors(event)?,
        }

        Ok(())
    }

    /// Обрабатывает все события, связанные с мышкой
    pub fn on_mouse_event(&mut self, _event: MouseEvent) -> std::io::Result<()> {
        // dbg!(event);
        Ok(())
    }

    /// Обновляет поля датчиков
    pub fn update_sensor_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Получаем новые поля и сохраняем их
        let sensor_fields = self.get_sensors_fields()?;
        *self.sensor_fields.borrow_mut() = sensor_fields;

        // Получаем новые серийники и сохраняем их
        let sensor_serials = self.get_sensors_serials()?;
        *self.sensor_serials.borrow_mut() = sensor_serials;

        // Обновляем данные датчиков во вкладках
        self.tabs.update_sensor_data();

        Ok(())
    }

    /// Открывает новую вкладку
    pub fn open_new_tab(&mut self) {
        // Создаём новую вкладку
        let mut graph_state =
            GraphState::new(self.sensor_fields.clone(), self.sensor_serials.clone());
        graph_state.update_sensor_data();

        // Открываем новую вкладку
        let app_state = TabState::Graph(graph_state);
        self.tabs.open(app_state);
    }

    /// Возвращает ссылку на активное состояние
    pub fn state(&self) -> &TabState<'a> {
        self.tabs.state()
    }

    /// Возвращает изменяемую ссылку на активное состояние
    pub fn state_mut(&mut self) -> &mut TabState<'a> {
        self.tabs.state_mut()
    }

    /// Возвращает ссылку на активное состояние вкладки графика
    pub fn graph_state(&self) -> &GraphState {
        self.state().graph().unwrap()
    }

    /// Возвращает изменяемую ссылку на активное состояние вкладки графика
    pub fn graph_state_mut(&mut self) -> &mut GraphState {
        self.state_mut().graph_mut().unwrap()
    }

    /// Возвращает ссылку на активное состояние вкладки дерева сенсоров
    pub fn sensors_state(&self) -> &SensorsState<'a> {
        self.state().sensors().unwrap()
    }

    /// Возвращает изменяемую ссылку на активное состояние вкладки дерева сенсоров
    pub fn sensors_state_mut(&mut self) -> &mut SensorsState<'a> {
        self.state_mut().sensors_mut().unwrap()
    }

    /// Возвращает ссылку на состояние окна выбора файла
    pub fn file_picker_state(&self) -> &FilePickerState {
        self.sensors_state().file_picker_state.as_ref().unwrap()
    }

    /// Возвращает изменяемую ссылку на состояние окна выбора файла
    pub fn file_picker_state_mut(&mut self) -> &mut FilePickerState {
        self.sensors_state_mut().file_picker_state.as_mut().unwrap()
    }
}
