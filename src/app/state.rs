use std::sync::{Arc, Mutex};

use crossterm::event::{KeyEvent, MouseEvent};

use crate::{
    filepicker::state::FilePickerState, graph::state::GraphState, sensors::state::SensorsState,
};

use super::tabs::Tabs;

/// Структура, определяющая состояние приложения
/// Используется везде, где только можно
#[derive(Debug)]
pub struct App<'a> {
    /// Определяет, работает ли сейчас программа
    pub running: bool,

    /// Соединение с базой данных
    pub database: Arc<Mutex<rusqlite::Connection>>,

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
            tabs: Tabs::default(),
        };

        // Подготавливаем первую вкладку - вкладка сенсоров
        let sensors_state = SensorsState::default();
        let app_state = AppState::Sensors(sensors_state);
        app.tabs.open(app_state);

        // Подготавливаем дерево сенсоров
        app.update_sensors_tree()?;

        Ok(app)
    }

    /// Выполняет один тик обновления в состоянии приложения
    pub fn tick(&mut self) {
        match self.tabs.state() {
            AppState::Graph(_) => self.tick_graph(),
            AppState::Sensors(_) => self.tick_sensors(),
        }
    }

    /// Обрабатывает все события, связанные с нажатием клавиш
    pub fn on_key_event(&mut self, event: KeyEvent) -> std::io::Result<()> {
        match self.tabs.state() {
            AppState::Graph(_) => self.on_key_event_graph(event),
            AppState::Sensors(_) => self.on_key_event_sensors(event)?,
        }

        Ok(())
    }

    /// Обрабатывает все события, связанные с мышкой
    pub fn on_mouse_event(&mut self, _event: MouseEvent) -> std::io::Result<()> {
        // dbg!(event);
        Ok(())
    }

    /// Открывает новую вкладку
    pub fn open_new_tab(&mut self) {
        let graph_state = GraphState::new();
        let app_state = AppState::Graph(graph_state);
        self.tabs.open(app_state);
    }

    /// Возвращает ссылку на активное состояние
    pub fn state(&self) -> &AppState<'a> {
        self.tabs.state()
    }

    /// Возвращает изменяемую ссылку на активное состояние
    pub fn state_mut(&mut self) -> &mut AppState<'a> {
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

/// Перечисляемый тип, определяющий режим приложения в данный момент
/// Используется для работы всяких окошечек и менюшечек
#[derive(Debug)]
pub enum AppState<'a> {
    /// Вкладка с графиком
    Graph(GraphState),

    /// Вкладка с деревом датчиков
    Sensors(SensorsState<'a>),
}

impl<'a> AppState<'a> {
    /// Возвращает ссылку на состояние графика
    pub fn graph(&self) -> Option<&GraphState> {
        match self {
            Self::Graph(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает ссылку на состояние дерева сенсоров
    pub fn sensors(&self) -> Option<&SensorsState<'a>> {
        match self {
            Self::Sensors(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает изменяемую ссылку на состояние графика
    pub fn graph_mut(&mut self) -> Option<&mut GraphState> {
        match self {
            Self::Graph(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает изменяемую ссылку на состояние дерева сенсоров
    pub fn sensors_mut(&mut self) -> Option<&mut SensorsState<'a>> {
        match self {
            Self::Sensors(state) => Some(state),
            _ => None,
        }
    }
}
