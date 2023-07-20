use std::sync::{Arc, Mutex};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::{filepicker::state::FilePickerState, ui::sensors_tab::SensorsTree};

use super::tabs::Tabs;

/// Структура, определяющая состояние приложения
/// Используется везде, где только можно
#[derive(Debug)]
pub struct App<'a> {
    /// Определяет, работает ли сейчас программа
    pub running: bool,

    /// Определяет, в каком режиме сейчас находится программа
    pub state: AppState,

    /// Соединение с базой данных
    pub database: Arc<Mutex<rusqlite::Connection>>,

    /// Определяет дерево сенсоров на главной вкладке
    pub sensors_tree: SensorsTree<'a>,

    /// Определяет вкладки, открытые в приложении
    pub tabs: Tabs,
}

impl<'a> App<'a> {
    /// Создаёт новое состояние приложения
    pub fn new(database: rusqlite::Connection) -> Result<Self, Box<dyn std::error::Error>> {
        // Делаем базовый экземпляр состояния приложения
        let mut app = Self {
            running: true,
            state: AppState::default(),

            database: Arc::new(Mutex::new(database)),
            sensors_tree: SensorsTree::default(),

            tabs: Tabs::default(),
        };

        // Подготавливаем дерево сенсоров
        app.update_sensors_tree()?;

        Ok(app)
    }

    /// Выполняет один тик обновления в состоянии приложения
    pub fn tick(&mut self) {
        match self.state {
            AppState::Default => self.tick_default(),
            AppState::FilePicker(_) => self.tick_filepicker(),
        }
    }

    /// Обрабатывает тик в обычном режиме приложения
    fn tick_default(&mut self) {}

    /// Обрабатывает все события, связанные с нажатием клавиш
    pub fn on_key_event(&mut self, event: KeyEvent) -> std::io::Result<()> {
        match self.state {
            AppState::Default => self.on_key_event_default(event)?,
            AppState::FilePicker(_) => self.on_key_event_filepicker(event)?,
        }

        Ok(())
    }

    /// Обрабатывает все события, связанные с нажатием клавиш в обычном режиме
    pub fn on_key_event_default(&mut self, event: KeyEvent) -> std::io::Result<()> {
        match event.code {
            KeyCode::BackTab => self.tabs.prev(),
            KeyCode::Tab => self.tabs.next(),

            KeyCode::Char('q') if event.modifiers == KeyModifiers::CONTROL => self.running = false,
            KeyCode::Char('i') => self.open_filepicker()?,

            KeyCode::Up => self.sensors_tree.up(),
            KeyCode::Down => self.sensors_tree.down(),
            KeyCode::Left => self.sensors_tree.left(),
            KeyCode::Right => self.sensors_tree.right(),

            _ => (),
        }

        Ok(())
    }

    /// Обрабатывает все события, связанные с мышкой
    pub fn on_mouse_event(&mut self, _event: MouseEvent) -> std::io::Result<()> {
        // dbg!(event);
        Ok(())
    }
}

/// Перечисляемый тип, определяющий режим приложения в данный момент
/// Используется для работы всяких окошечек и менюшечек
#[derive(Default, Debug)]
pub enum AppState {
    /// Обычное состояние
    #[default]
    Default,

    /// Файловый менеджер, для выбора файла для импорта
    FilePicker(FilePickerState),
}

impl AppState {
    /// Функция для получения состояния выбора файла
    pub fn file_picker_state(&mut self) -> Option<&mut FilePickerState> {
        match self {
            Self::FilePicker(state) => Some(state),
            _ => None,
        }
    }
}
