use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::Paragraph,
    Frame,
};
use tui_tree_widget::Tree;

use crate::filepicker::ui::draw_file_picker;

use super::state::SensorsState;

/// Рендерит вкладку с деревом датчиков
pub fn draw_sensors_tab<B: Backend>(frame: &mut Frame<B>, state: &mut SensorsState, area: Rect) {
    // Если у нас есть данные датчиков, рисуем дерево
    if !state.items.is_empty() {
        let tree = Tree::new(state.items.clone()).highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_stateful_widget(tree, area, &mut state.tree_state);
    } else {
        let text = "--- Данные датчиков не импортированы ---";
        let paragraph = Paragraph::new(vec![Spans::from(text)]);
        frame.render_widget(paragraph, area);
    }

    // Если у нас открыто окно выбора файла -> рендерим его
    if let Some(file_picker_state) = state.file_picker_state.as_mut() {
        draw_file_picker(frame, file_picker_state, area);
    }
}
