use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    graph::state::GraphFieldState,
    ui::menu::{Menu, MENU_HEIGHT},
};

use super::state::GraphState;

/// Рендерит вкладку с графиком
pub fn draw_graph_tab<B: Backend>(frame: &mut Frame<B>, state: &mut GraphState, area: Rect) {
    // Разделяем фрейм на части
    let area_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // 1 клетка для рамки, 1 клетка для подписей полей, 1 клетка для X
            Constraint::Length(3 + state.ys_states.len() as u16),
            Constraint::Min(0),
        ])
        .split(area);

    // Рендерим графики (необходимо, чтобы они не перекрывали возможные меню)
    let chart_area = area_chunks[1];
    let text = Text::raw("Тут должен быть график");
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, chart_area);

    // Делаем блок для полей графика
    let fields_area = area_chunks[0];
    let fields_block = Block::default().borders(Borders::BOTTOM);
    // Рендерим нижнюю черту
    frame.render_widget(fields_block, fields_area);
    // Рендерим поля
    draw_graph_fields(frame, state, fields_area);
}

/// Рендерит поля графика
fn draw_graph_fields<B: Backend>(frame: &mut Frame<B>, state: &mut GraphState, area: Rect) {
    // Делаем текст названий переменных и функций
    let mut fields = vec!["X:".to_owned()];
    fields.extend(
        state
            .ys_states
            .iter()
            .enumerate()
            .map(|(i, _)| format!("Y{}(x):", i + 1)),
    );
    // Узнаём максимальную длину названия функции
    let max_length = fields.last().unwrap().len();

    // Делим данную область на поля
    let area_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            // 1 клетка для рамки, 1 клетка для X
            Constraint::Length(max_length as u16 + 2),
            Constraint::Min(0),
        ])
        .split(area);

    // Обрабатываем область под названия переменных
    let mut var_names_area = area_chunks[0];
    var_names_area.y += 1;
    var_names_area.height -= 1;

    // Рендерим названия переменных
    let spans: Vec<_> = fields.into_iter().map(Spans::from).collect();
    let paragraph = Paragraph::new(spans);
    frame.render_widget(paragraph, var_names_area);

    // Если находимся в режиме редактирования полей, отображаем надпись
    if state.selected.is_some() {
        // Подготавливаем область для написания этого
        let area = Rect {
            height: 1,
            ..area_chunks[0]
        };

        // Рендерим надпись
        let paragraph = Paragraph::new(Text::raw("Edit"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        frame.render_widget(paragraph, area);
    }

    // Подготавливаем области для каждого параметра
    let fields_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25); 4])
        .split(area_chunks[1]);

    // Рендерим названия параметров
    const NAMES: [&str; 4] = [
        "Поле данных",
        "Серийник",
        "Мин.знач./Датчик",
        "Макс.знач./Датчик",
    ];
    for (i, name) in NAMES.into_iter().enumerate() {
        // Получаем область для рендера
        let area = fields_areas[i];

        // Рендерим надпись
        let paragraph = Paragraph::new(vec![Spans::from(name)]).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    let fields: Vec<_> = state.sensor_fields.iter().cloned().map(Text::raw).collect();

    // Подготавливаем итераторы
    let x_states = (0, &mut state.x_states);
    let ys_states = state
        .ys_states
        .iter_mut()
        .enumerate()
        .map(|(i, state)| (i + 1, state))
        .rev();

    // Рендерим каждое поле (необходимо делать в обратном порядке, чтобы не было пересечений с меню)
    for (i, variable) in ys_states.chain(std::iter::once(x_states)) {
        for (j, field) in variable.iter_mut().enumerate() {
            // Проверяем, какое поле нам надо рендерить
            match field {
                // Рендерим меню
                GraphFieldState::Menu(menu_state) => {
                    // Получаем общую область
                    let area = fields_areas[j];

                    // Получаем область под поле
                    let field_area = Rect {
                        y: area.y + i as u16 + 1,
                        width: area.width - 1,
                        height: 1 + MENU_HEIGHT, // 1 для названия
                        ..area
                    };

                    // Подготавливаем меню
                    let menu = Menu::new(fields.clone())
                        .list_style(Style::default().bg(Color::White).fg(Color::Black))
                        .list_highlight_style(Style::default().bg(Color::Green).fg(Color::Black));

                    // Делаем выбранный элемент выделенным, если меню "выделено"
                    let menu = if state.selected == Some(i * 4 + j) {
                        menu.style(Style::default().fg(Color::Green))
                    } else {
                        menu
                    };

                    // Рендерим меню
                    frame.render_stateful_widget(menu, field_area, menu_state);
                }
                // Рендерим поле ввода текста
                GraphFieldState::Input(_) => {
                    // Получаем общую область
                    let area = fields_areas[j];
                    let area = Rect::new(area.x, area.y + i as u16 + 1, area.width - 1, 1);

                    // Подготавливаем стиль для элемента
                    let style = Style::default().add_modifier(Modifier::UNDERLINED);
                    // Делаем выбранный элемент выделенным, если меню "выделено"
                    let style = if state.selected == Some(i * 4 + j) {
                        style.fg(Color::Green)
                    } else {
                        style
                    };

                    // TODO: Рендерим поле
                    let paragraph = Paragraph::new(vec![Spans::from("Инпут")]).style(style);
                    frame.render_widget(paragraph, area);
                }
                // Пустые поля не надо рендерить
                GraphFieldState::Hidden => (),
            }
        }
    }
}
