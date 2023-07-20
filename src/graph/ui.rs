use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::state::GraphState;

/// Рендерит вкладку с графиком
pub fn draw_graph_tab<B: Backend>(frame: &mut Frame<B>, state: &mut GraphState, area: Rect) {
    // Разделяем фрейм на части
    let frame_chunks = Layout::default()
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(area);

    // Делаем блок для полей графика
    let fields_block = Block::default().borders(Borders::BOTTOM);
    // Рендерим нижнюю черту
    frame.render_widget(fields_block, frame_chunks[0]);
    // Рендерим поля
    draw_graph_fields(frame, state, frame_chunks[0]);

    let text = Text::raw("Тут должен быть график");
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, frame_chunks[1]);
}

/// Рендерит поля графика
fn draw_graph_fields<B: Backend>(frame: &mut Frame<B>, _state: &mut GraphState, area: Rect) {
    let text = Text::raw("Поле X\nПоле Y\nЕщё одно поле Y\n");
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}
