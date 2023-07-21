use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::{
    app::{state::App, tabs::TabState},
    graph::ui::draw_graph_tab,
    sensors::ui::draw_sensors_tab,
};

pub mod menu;
pub mod utils;

/// Основная функция рендера интерфейса
pub fn draw<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    // Разделяем фрейм на части
    let frame_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.size());

    // Превращаем названия вкладок в нужный формат
    let tabs_titles = app
        .tabs
        .titles()
        .into_iter()
        .map(|title| Spans::from(Span::raw(title)))
        .collect();
    // Делаем виджет вкладок и рендерим его
    let tabs_block = Block::default()
        .borders(Borders::BOTTOM)
        .title("Практика")
        .title_alignment(Alignment::Center);
    let tabs = Tabs::new(tabs_titles)
        .block(tabs_block)
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.current);
    frame.render_widget(tabs, frame_chunks[0]);

    // Рендерим вкладку
    let main_area = frame_chunks[1];
    match app.state_mut() {
        TabState::Graph(state) => draw_graph_tab(frame, state, main_area),
        TabState::Sensors(state) => draw_sensors_tab(frame, state, main_area),
    }
}
