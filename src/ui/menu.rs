use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Clear, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

/// Описывает данное состояние виджета "меню"
#[derive(Default, Debug)]
pub struct MenuState {
    /// Определяет то, какой элемент сейчас выбран
    selected: Option<usize>,

    /// Хранит состояние списка элементов, когда он открыт
    list_state: ListState,
}

impl MenuState {
    /// Открывает меню
    pub fn open(&mut self) {
        self.list_state.select(Some(0));
    }

    /// Закрывает меню
    pub fn close(&mut self) {
        self.list_state.select(None);
    }

    /// Возвращает, открыто ли меню
    pub fn opened(&self) -> bool {
        self.list_state.selected().is_some()
    }

    /// Переходит к предыдущему элементу меню
    pub fn prev(&mut self, length: usize) {
        let i = self.list_state.selected().unwrap();
        self.list_state.select(Some((i + length - 1) % length));
    }

    /// Переходит к следующему элементу меню
    pub fn next(&mut self, length: usize) {
        let i = self.list_state.selected().unwrap();
        self.list_state.select(Some((i + 1) % length));
    }

    /// Выбирает элемент из меню
    pub fn select(&mut self) {
        self.selected = self.list_state.selected();
        self.close();
    }

    /// Возвращает выбранный элемент
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }
}

/// Представляет из себя виджет "меню"
#[derive(Debug)]
pub struct Menu<'a> {
    /// Сохранённый список элементов для отображения
    items: Vec<Text<'a>>,

    /// Список для выбора вариантов меню
    list: List<'a>,

    /// Стиль того, как выглядит выбранный элемент
    style: Style,
}

impl<'a> Menu<'a> {
    /// Создаёт новый виджет меню
    pub fn new<T>(items: T) -> Self
    where
        T: Into<Vec<Text<'a>>>,
    {
        let items: Vec<Text<'a>> = items.into();
        let list_items: Vec<ListItem<'a>> = items.iter().cloned().map(ListItem::new).collect();

        Self {
            items,
            list: List::new(list_items),

            style: Style::default(),
        }
    }

    /// Задаёт стиль выбранному значению
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Задаёт стиль меню
    pub fn list_style(mut self, style: Style) -> Self {
        self.list = self.list.style(style);
        self
    }

    /// Задаёт стиль выделенному элементу меню
    pub fn list_highlight_style(mut self, style: Style) -> Self {
        self.list = self.list.highlight_style(style);
        self
    }
}

/// Определяет сколько строк может максимально занимать меню
pub const MENU_HEIGHT: u16 = 5;

impl StatefulWidget for Menu<'_> {
    type State = MenuState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Рендерим выбранную строку
        let text = if let Some(selected) = state.selected {
            self.items.swap_remove(selected)
        } else {
            Text::raw("<значение не выбрано>")
        };
        let field_area = Rect { height: 1, ..area };
        let paragraph = Paragraph::new(text).style(self.style);
        paragraph.render(field_area, buf);

        // Рендер символа в конце выбранного элемента, для отображения состояния меню
        let opened_area = Rect {
            x: field_area.x + field_area.width - 2,
            width: 2,
            ..field_area
        };

        // Выбор символа в зависимости от состояния
        let text = Text::raw(if state.opened() {
            " \u{25bc}"
        } else {
            " \u{25c0}"
        });

        // Рендер символа
        let paragraph = Paragraph::new(text);
        paragraph.render(opened_area, buf);

        // Рендерим окно меню, если оно открыто
        if state.opened() {
            // Подготавливаем область для меню
            let area = Rect {
                y: area.y + 1,
                height: MENU_HEIGHT,
                ..area
            };

            // Вычищаем область перед рендером списка
            Widget::render(Clear, area, buf);

            // Рендерим список полей (т.е. меню)
            StatefulWidget::render(self.list, area, buf, &mut state.list_state);
        }
    }
}
