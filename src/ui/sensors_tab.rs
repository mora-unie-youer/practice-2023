use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::Paragraph,
    Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::app::state::App;

impl App<'_> {
    /// Возвращает массив элементов для дерева полей сенсора
    pub fn update_sensors_tree(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Получаем поля сенсоров для дерева
        let sensors_fields = self.get_sensors_fields()?;

        // Преобразуем поля в дерево
        let sensors_tree = sensors_fields
            .into_iter()
            .map(|(name, fields)| {
                TreeItem::new(
                    name,
                    fields
                        .into_iter()
                        .map(TreeItem::new_leaf)
                        .collect::<Vec<_>>(),
                )
            })
            .collect();

        self.sensors_tree.items = sensors_tree;
        Ok(())
    }
}

/// Определяет дерево датчиков
#[derive(Default, Debug)]
pub struct SensorsTree<'a> {
    /// Хранит состояние дерево, полученное крейтом
    pub state: TreeState,

    /// Хранит элементы дерева
    pub items: Vec<TreeItem<'a>>,
}

impl<'a> SensorsTree<'a> {
    /// Отправляет модулю Tree нажатие кнопки "вниз"
    pub fn down(&mut self) {
        self.state.key_down(&self.items);
    }

    /// Отправляет модулю Tree нажатие кнопки "вверх"
    pub fn up(&mut self) {
        self.state.key_up(&self.items);
    }

    /// Отправляет модулю Tree нажатие кнопки "влево"
    pub fn left(&mut self) {
        self.state.key_left();
    }

    /// Отправляет модулю Tree нажатие кнопки "вправо"
    pub fn right(&mut self) {
        self.state.key_right();
    }
}

// Рендерит вкладку с данными
pub fn draw_sensors_tab<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    // Если у нас есть данные датчиков, рисуем дерево
    if !app.sensors_tree.items.is_empty() {
        let tree = Tree::new(app.sensors_tree.items.clone())
            .highlight_style(
                Style::default()
                    .fg(Color::Indexed(2))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        frame.render_stateful_widget(tree, area, &mut app.sensors_tree.state);
    } else {
        let text = "--- Данные датчиков не импортированы ---";
        let paragraph = Paragraph::new(vec![Spans::from(text)]);
        frame.render_widget(paragraph, area);
    }
}
