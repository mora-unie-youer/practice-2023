use crossterm::event::{KeyCode, KeyEvent};

use crate::app::state::App;

pub mod state;
pub mod ui;

impl App<'_> {
    /// Выполняет один тик обновления во вкладке графика
    pub fn tick_graph(&mut self) {}

    /// Обрабатывает все события, связанные с нажатием клавиш во вкладке графика
    pub fn on_key_event_graph(&mut self, event: KeyEvent) {
        match event.code {
            // Управление вкладками
            KeyCode::BackTab => self.tabs.prev(),
            KeyCode::Tab => self.tabs.next(),
            KeyCode::Char('N') => self.open_new_tab(),
            KeyCode::Char('q') => self.tabs.close(),

            _ => (),
        }
    }
}
