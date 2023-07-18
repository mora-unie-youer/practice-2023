use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

/// Структура, определяющая состояние приложения
/// Используется везде, где только можно
pub struct App {
    pub running: bool,
    pub state: AppState,

    pub tabs: Tabs,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Создаёт новое состояние приложения
    pub fn new() -> Self {
        Self {
            running: true,
            state: AppState::default(),
            tabs: Tabs::default(),
        }
    }

    /// Выполняет один тик обновления в состоянии приложения
    pub fn tick(&mut self) {}

    /// Обрабатывает все события, связанные с нажатием клавиш
    pub fn on_key_event(&mut self, event: KeyEvent) {
        match self.state {
            AppState::Default => self.on_key_event_default(event),
            AppState::FilePicker => self.on_key_event_filepicker(event),
        }
    }

    /// Обрабатывает все события, связанные с нажатием клавиш в обычном режиме
    pub fn on_key_event_default(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::BackTab => self.tabs.prev(),
            KeyCode::Tab => self.tabs.next(),

            KeyCode::Char('q') if event.modifiers == KeyModifiers::CONTROL => self.running = false,
            KeyCode::Char('i') => self.state = AppState::FilePicker,

            _ => (),
        }
    }

    /// Обрабатывает все события, связанные с мышкой
    pub fn on_mouse_event(&mut self, event: MouseEvent) {
        // dbg!(event);
    }
}

/// Перечисляемый тип, определяющий режим приложения в данный момент
/// Используется для работы всяких окошечек и менюшечек
#[derive(Default)]
pub enum AppState {
    /// Обычное состояние
    #[default]
    Default,

    /// Файловый менеджер, для выбора файла для импорта
    FilePicker,
}

/// Структура определяющая состояние вкладок
/// Используется для определения того, в какой вкладке мы находимся, и что должны отображать
pub struct Tabs {
    pub titles: Vec<String>,
    pub current: usize,
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Tabs {
    /// Создаёт новое состояние вкладок
    pub fn new() -> Self {
        Self {
            // Первая вкладка всегда данные и не может быть закрыта
            titles: vec!["Данные".into(), "График 1".into(), "График 2".into()],
            current: 0,
        }
    }

    /// Переключает на следующую вкладку
    pub fn next(&mut self) {
        self.current = (self.current + 1) % self.titles.len();
    }

    /// Переключает на предыдущую вкладку
    pub fn prev(&mut self) {
        self.current = (self.current + self.titles.len() - 1) % self.titles.len();
    }
}
