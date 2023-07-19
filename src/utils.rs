use tui::layout::{Constraint, Direction, Layout, Rect};

/// Возвращает область выделенную под popup
pub fn get_popup_area(percent_width: u16, percent_height: u16, area: Rect) -> Rect {
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

/// Возвращает внутреннюю область блока
pub fn get_inner_block_area(area: Rect) -> Rect {
    Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2)
}
