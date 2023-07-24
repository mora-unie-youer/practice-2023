use std::{cell::RefCell, collections::HashMap, rc::Rc};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Span, Spans, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::{
    graph::state::GraphFieldState,
    ui::{
        input::Input,
        menu::{Menu, MENU_HEIGHT},
    },
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
    draw_graph_chart(frame, state, chart_area);

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

    // Подготавливаем данные для рендера
    let x_data_fields: Vec<_> = state.x_data_fields.iter().cloned().map(Text::raw).collect();
    let y_data_fields: Vec<_> = state.y_data_fields.iter().cloned().map(Text::raw).collect();
    let y_data_fields_without_extra: HashMap<_, Vec<_>> = state
        .y_data_fields_without_extra
        .clone()
        .into_iter()
        .map(|(sensor, fields)| {
            let fields = fields.iter().cloned().map(Text::raw).collect();
            (sensor.clone(), fields)
        })
        .collect();
    let serial_fields: HashMap<_, Vec<_>> = state
        .serial_fields
        .iter()
        .map(|(sensor, fields)| {
            let fields = fields.iter().cloned().map(Text::raw).collect();
            (sensor.clone(), fields)
        })
        .collect();

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
        // Сохранем выбор поля данных для будущих действий
        let data_selection = variable[0].menu().unwrap().selected();

        for (j, field) in variable.iter_mut().enumerate() {
            // Делаем выбранный элемент выделенным, если меню "выделено"
            let style = if state.selected == Some(i * 4 + j) {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

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

                    // Получаем элементы меню
                    let items = match j {
                        // Поля значений X
                        0 if i == 0 => x_data_fields.clone(),
                        // Поля значений Y
                        0 => y_data_fields.clone(),
                        // Поля серийников (поле значений гарантированно выбрано)
                        1 => {
                            // Получаем выбор с поля данных
                            let data_selection = data_selection.unwrap();
                            // В зависимости от переменной, сенсор надо получать из разных массивов
                            if i == 0 {
                                // Обрабатываем как поле X
                                let selection = &state.x_data_fields[data_selection];
                                let (sensor, _) = selection.split_once('/').unwrap();
                                // Получаем серийники датчика
                                serial_fields[sensor].clone()
                            } else {
                                // Обрабатываем как поле Y
                                let selection = &state.y_data_fields[data_selection];
                                let (sensor, _) = selection.split_once('/').unwrap();
                                // Получаем серийники датчика
                                serial_fields[sensor].clone()
                            }
                        }

                        // Поля дополнительных данных Y
                        _ => {
                            // Получаем выбор с поля данных
                            let data_selection = data_selection.unwrap();
                            // Получаем название сенсора
                            let selection = &state.y_data_fields[data_selection];
                            let (sensor, _) = selection.split_once('/').unwrap();
                            // Получаем поля необходимые для отображения
                            y_data_fields_without_extra[sensor].clone()
                        }
                    };

                    // Подготавливаем меню
                    let menu = Menu::new(items)
                        .list_style(Style::default().bg(Color::White).fg(Color::Black))
                        .list_highlight_style(Style::default().bg(Color::Green).fg(Color::Black))
                        .style(style);

                    // Рендерим меню
                    frame.render_stateful_widget(menu, field_area, menu_state);
                }
                // Рендерим поле ввода текста
                GraphFieldState::Input(input_state) => {
                    // Получаем общую область
                    let area = fields_areas[j];
                    let area = Rect::new(area.x, area.y + i as u16 + 1, area.width - 1, 1);

                    // Подготавливаем стиль для элемента
                    let style = style.add_modifier(Modifier::UNDERLINED);

                    // Рендерим поле
                    let input = Input::new().style(style);
                    frame.render_stateful_widget(input, area, input_state);
                }
                // Пустые поля не надо рендерить
                GraphFieldState::Hidden => (),
            }
        }
    }
}

/// Рендерит общий чарт со всеми графиками по датасетам в состоянии
fn draw_graph_chart<B: Backend>(frame: &mut Frame<B>, state: &mut GraphState, area: Rect) {
    // Перерисовываем чарт только когда не редактируем его, иначе это невыносимые лаги
    if state.selected.is_some() {
        return;
    }

    const GRAPH_COLORS: [Color; 7] = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
    ];

    // Собираем датасеты
    let color_iter = Rc::new(RefCell::new(GRAPH_COLORS.into_iter().cycle()));
    let datasets = state
        .datasets
        .iter()
        .enumerate()
        .flat_map(|(i, y_datasets)| {
            let color_iter = color_iter.clone();
            y_datasets.iter().enumerate().map(move |(j, dataset)| {
                Dataset::default()
                    .name(format!("Y{}.{}", i + 1, j + 1))
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(color_iter.borrow_mut().next().unwrap()))
                    .data(dataset)
            })
        })
        .collect();

    // Создаём виджет чарта
    let (x_range, y_range) = state.dataset_ranges;
    let chart = Chart::new(datasets)
        .hidden_legend_constraints((Constraint::Ratio(1, 4), Constraint::Ratio(1, 1)))
        .x_axis(
            Axis::default()
                .title(Span::styled("X", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds(x_range.into())
                .labels(
                    [x_range.0.to_string(), x_range.1.to_string()]
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Y", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds(y_range.into())
                .labels(
                    [y_range.0.to_string(), y_range.1.to_string()]
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        );

    // Рендерим
    frame.render_widget(chart, area);
}
