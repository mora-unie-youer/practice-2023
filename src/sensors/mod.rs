use crossterm::event::{KeyCode, KeyEvent};

use crate::app::state::App;

pub mod state;
pub mod ui;

impl App<'_> {
    /// Выполняет один тик обновления во вкладке дерева сенсоров
    pub fn tick_sensors(&mut self) {
        // Получаем состояние дерева сенсоров
        let state = self.sensors_state_mut();

        // Проверяем открыто ли окно выбора файла
        if state.file_picker_state.is_some() {
            self.tick_file_picker();
        }
    }

    /// Обрабатывает все события, связанные с нажатием клавиш во вкладке дерева сенсоров
    pub fn on_key_event_sensors(&mut self, event: KeyEvent) -> std::io::Result<()> {
        // Получаем состояние дерева сенсоров
        let state = self.sensors_state_mut();

        // Проверяем открыто ли окно выбора файла
        if state.file_picker_state.is_some() {
            self.on_key_event_file_picker(event)?;
        } else {
            match event.code {
                // Выход из приложения
                KeyCode::Char('q') => self.running = false,
                // Управление вкладками
                KeyCode::BackTab => self.tabs.prev(),
                KeyCode::Tab => self.tabs.next(),
                KeyCode::Char('N') => self.open_new_tab(),
                // Управление деревом
                KeyCode::Up => state.tree_state.key_up(&state.items),
                KeyCode::Down => state.tree_state.key_down(&state.items),
                KeyCode::Left => state.tree_state.key_left(),
                KeyCode::Right => state.tree_state.key_right(),
                // Импорт данных
                KeyCode::Char('i') => self.open_file_picker()?,

                _ => (),
            }
        }

        Ok(())
    }
}
