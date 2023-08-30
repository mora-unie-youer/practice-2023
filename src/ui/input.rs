use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Paragraph, StatefulWidget, Widget},
};

/// Описывает данное состояние виджета ввода текста
#[derive(Default, Debug)]
pub struct InputState {
    content: String,
    offset: usize,
    cursor: Option<usize>,
}

impl InputState {
    /// Возвращает содержимое поля ввода
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Открывает поле ввода
    pub fn open(&mut self) {
        self.cursor = Some(self.content.len());
    }

    /// Закрывает поле ввода
    pub fn close(&mut self) {
        self.cursor = None;
    }

    /// Возвращает, открыто ли поле ввода
    pub fn opened(&self) -> bool {
        self.cursor.is_some()
    }

    /// Передаёт ввод в поле
    pub fn insert(&mut self, ch: char) {
        if let Some(i) = self.cursor.as_mut() {
            self.content.insert(*i, ch);
            *i += 1;
        }
    }

    /// Удаляет выбранный символ
    pub fn remove(&mut self) {
        if let Some(i) = self.cursor.as_mut() {
            if *i > 0 {
                self.content.remove(*i - 1);
                self.offset = self.offset.saturating_sub(1);
                *i -= 1;
            }
        }
    }

    /// Переходит к предыдущему символу, если это возможно
    pub fn goto_prev(&mut self) {
        if let Some(i) = self.cursor.as_mut() {
            *i = i.saturating_sub(1);
        }
    }

    /// Переходит к следующему символу, если это возможно
    pub fn goto_next(&mut self) {
        if let Some(i) = self.cursor.as_mut() {
            *i = self.content.len().min(*i + 1);
        }
    }

    /// Переходит к началу ввода
    pub fn goto_start(&mut self) {
        if let Some(i) = self.cursor.as_mut() {
            *i = 0;
        }
    }

    /// Переходит к началу ввода
    pub fn goto_end(&mut self) {
        if let Some(i) = self.cursor.as_mut() {
            *i = self.content.len();
        }
    }

    /// Возвращает границы списка файлов, которые необходимо рендерить
    pub fn get_render_bounds(&mut self, max_width: usize) -> (usize, usize) {
        // Если строка пустая, возвращаем пустую область
        if self.content.is_empty() {
            return (0, 0);
        }

        // Получаем элементы для более удобного доступа
        let content = &self.content;

        // Подготавливаем отступ, начало, конец, ширину
        let offset = self.offset.min(content.len().saturating_sub(1));
        let mut start = offset;
        let mut width = max_width.min(content.len() - offset);
        let mut end = offset + width;

        // Получаем положение курсора на данный момент (учитывая что оно на один больше чем длина строки)
        let cursor = self.cursor.unwrap_or(1).saturating_sub(1);
        // Сдвигаемся, пока мы не имеем выделение в правой границе
        while cursor >= end {
            end += 1;
            width += 1;
            if width > max_width {
                start += 1;
                width -= 1;
            }
        }

        // Сдвигаемся, пока не имеем выделение в левой границе
        while cursor < start {
            start -= 1;
            width += 1;
            if width > max_width {
                end -= 1;
                width -= 1;
            }
        }

        self.offset = start;
        (start, end)
    }
}

/// Представляет из себя виджет ввода текста
#[derive(Default, Debug)]
pub struct Input {
    /// Стиль того, как выглядит поле
    style: Style,
}

impl Input {
    /// Создаёт новый виджет ввода текста
    pub fn new() -> Self {
        Self {
            style: Style::default(),
        }
    }

    /// Задаёт стиль того, как должно выглядеть поле
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl StatefulWidget for Input {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Получаем границы рендера
        let max_width = area.width as usize - 1;
        let (start, end) = state.get_render_bounds(max_width);

        // Рендерим введённый текст
        let content = &state.content[start..end];
        let paragraph = Paragraph::new(Text::raw(content)).style(self.style);
        paragraph.render(area, buf);

        // Рендерим курсорчик, если поле редактируется
        if let Some(cursor) = state.cursor {
            let relative_position = cursor - state.offset;
            let cursor_area = Rect {
                x: area.x + relative_position as u16,
                width: 1,
                ..area
            };

            buf.set_style(cursor_area, Style::default().bg(Color::White));
        }
    }
}
