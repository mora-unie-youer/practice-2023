use tui::{backend::Backend, layout::Rect, text::Spans, widgets::Paragraph, Frame};

use crate::app::state::App;

// Рендерит вкладку с графиком
pub fn draw_graph_tab<B: Backend>(frame: &mut Frame<B>, _app: &mut App, area: Rect) {
    let text = "Здесь должен быть график";
    let paragraph = Paragraph::new(vec![Spans::from(text)]);
    frame.render_widget(paragraph, area);
}
