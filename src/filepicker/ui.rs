use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::ui::utils::{get_inner_block_area, get_popup_area};

use super::state::{FilePickerItem, FilePickerState};

/// Рендерит окно выбора файла
pub fn draw_file_picker<B: Backend>(frame: &mut Frame<B>, state: &mut FilePickerState, area: Rect) {
    // Выделяем область под окошко
    let popup_area = get_popup_area(90, 80, area);

    // Делаем блок
    let block = Block::default()
        .title("Выбор файла для импорта")
        .borders(Borders::ALL);

    // Рендерим блок
    frame.render_widget(block, popup_area);

    // Рендерим список файлов
    draw_file_list(frame, state, popup_area);

    // Если у нас сейчас происходит момент импорта файла, отображаем окошко и ждём.
    if !state.import_threads.is_empty() {
        // Выделяем новую область под окошко
        let wait_popup_area = get_popup_area(30, 15, area);

        // Делаем блок и рендерим его
        let block = Block::default().borders(Borders::ALL);
        frame.render_widget(block, wait_popup_area);

        // Делаем виджет для рендера внутри
        let inner_area = get_inner_block_area(wait_popup_area);
        let text = "Данные импортируются, подождите...";
        let paragraph = Paragraph::new(Text::from(text)).alignment(Alignment::Center);
        frame.render_widget(paragraph, inner_area);
    }
}

/// Рендерит список файлов
fn draw_file_list<B: Backend>(frame: &mut Frame<B>, state: &mut FilePickerState, area: Rect) {
    // Выделяем область под список файлов
    let inner_area = get_inner_block_area(area);

    // Очищаем область рендера, чтобы не видеть артефакты
    frame.render_widget(Clear, inner_area);

    // Если директория не пустая -> обрабатываем файлы
    if !state.directory_contents.is_empty() {
        // Набор названий файлов и директорий
        let mut filenames: Vec<_> = state
            .directory_contents
            .iter()
            .map(|item| match item {
                FilePickerItem::File(f) => {
                    f.file_name().unwrap().to_os_string().into_string().unwrap()
                }
                FilePickerItem::Directory(f) => {
                    format!("{}/", f.file_name().unwrap().to_str().unwrap())
                }
            })
            .map(Span::raw)
            .collect();

        // Делаем выбранный файл выделенным
        filenames[state.selection_index].style = Style::default()
            .fg(Color::Indexed(2))
            .add_modifier(Modifier::BOLD);

        // Получаем границы рендера
        let (start, end) = state.get_render_bounds(inner_area.height as usize);

        // Делаем Spans для нормального отображения
        let filenames_spans: Vec<_> = filenames
            .into_iter()
            .skip(start)
            .take(end - start)
            .map(Spans::from)
            .collect();

        // Подготавливаем виджет для отображения
        let paragraph = Paragraph::new(filenames_spans);
        frame.render_widget(paragraph, inner_area);
    } else {
        // Делаем виджет для рендера внутри
        let text = "--- Директория пуста --";
        let paragraph = Paragraph::new(Text::from(text));
        frame.render_widget(paragraph, inner_area);
    }
}
