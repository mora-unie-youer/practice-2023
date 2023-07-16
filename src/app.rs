use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

/// Структура, определяющая состояние приложения
/// Используется везде, где только можно
pub struct App {
    pub running: bool,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    /// Создаёт новое состояние приложения
    pub fn new() -> Self {
        Self { running: true }
    }

    /// Выполнить один тик обновления в состоянии приложения
    pub fn tick(&mut self) {}

    /// Обрабатывает все события, связанные с нажатием клавиш
    pub fn on_key_event(&mut self, event: KeyEvent) {
        if event.modifiers == KeyModifiers::CONTROL && event.code == KeyCode::Char('q') {
            self.running = false;
        }
    }

    /// Обрабатывает все события, связанные с мышкой
    pub fn on_mouse_event(&mut self, event: MouseEvent) {}
}
