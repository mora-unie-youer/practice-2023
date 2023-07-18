use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::app::{App, AppState};

impl App {
    /// Обрабатывает все события, связанные с нажатием клавиш в режиме выбора файла
    pub fn on_key_event_filepicker(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => self.state = AppState::Default,
            _ => (),
        }
    }
}

fn get_popup_area(percent_width: u16, percent_height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_height) / 2),
                Constraint::Percentage(percent_height),
                Constraint::Percentage((100 - percent_height) / 2),
            ]
            .as_ref(),
        )
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_width) / 2),
                Constraint::Percentage(percent_width),
                Constraint::Percentage((100 - percent_width) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn draw_file_picker<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    // Выделяем область под окошко
    let popup_area = get_popup_area(90, 80, area);

    // Делаем блок
    let block = Block::default()
        .title("Выбор файла для импорта")
        .borders(Borders::ALL);

    // Рендерим блок
    frame.render_widget(block, popup_area);
}
