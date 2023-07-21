use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{List, ListItem, ListState, StatefulWidget},
};

/// Описывает данное состояние виджета "меню"
#[derive(Debug)]
pub struct MenuState {
    /// Определяет, открыто ли меню (другими словами, список под полем)
    opened: bool,

    /// Определяет то, какой элемент сейчас выбран
    selected: Option<usize>,

    /// Хранит состояние списка элементов, когда он открыт
    list_state: ListState,
}

/// Представляет из себя виджет "меню"
#[derive(Debug)]
pub struct Menu<'a> {
    /// Элементы меню
    list: List<'a>,
}

impl<'a> Menu<'a> {
    /// Создаёт новый виджет меню
    pub fn new<T>(items: T) -> Self
    where
        T: Into<Vec<ListItem<'a>>>,
    {
        Self {
            list: List::new(items),
        }
    }
}

impl StatefulWidget for Menu<'_> {
    type State = MenuState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // TODO: Рендерим выбранную строку

        // TODO: Рендерим окно меню, если оно открыто
        if state.opened {}
    }
}
