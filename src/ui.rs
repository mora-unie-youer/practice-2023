use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::{
    app::{App, AppState},
    filepicker::draw_file_picker,
};

/// Основная функция рендера интерфейса
pub fn draw<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    // Разделяем фрейм на части
    let frame_chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.size());

    // Превращаем названия вкладок в нужный формат
    let tabs_titles = app
        .tabs
        .titles
        .iter()
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
    match app.tabs.current {
        0 => draw_data_tab(frame, app, main_area),
        _ => draw_graph_tab(frame, app, main_area),
    }

    // Если у нас необычный режим работы программы, мы должны зарендерить это "особенное"
    match app.state {
        AppState::FilePicker => draw_file_picker(frame, app, main_area),
        AppState::Default => (),
    }
}

// Рендерит вкладку с данными
fn draw_data_tab<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    let text = "Здесь должны быть данные";
    let paragraph = Paragraph::new(vec![Spans::from(text)]);
    frame.render_widget(paragraph, area);
}

// Рендерит вкладку с графиком
fn draw_graph_tab<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    let text = "Здесь должен быть график";
    let paragraph = Paragraph::new(vec![Spans::from(text)]);
    frame.render_widget(paragraph, area);
}
