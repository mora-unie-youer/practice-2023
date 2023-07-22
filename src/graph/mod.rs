use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::state::App;

use self::state::{GraphFieldState, GraphState};

pub mod state;
pub mod ui;

impl App<'_> {
    /// Выполняет один тик обновления во вкладке графика
    pub fn tick_graph(&mut self) {}

    /// Обрабатывает все события, связанные с нажатием клавиш во вкладке графика
    pub fn on_key_event_graph(&mut self, event: KeyEvent) {
        // Получаем состояние вкладки графика
        let state = self.graph_state();

        if state.selected.is_some() {
            match state.selected_field_state() {
                GraphFieldState::Input(state) if state.opened() => {
                    self.on_key_event_graph_input(event);
                }
                GraphFieldState::Menu(state) if state.opened() => {
                    self.on_key_event_graph_menu(event);
                }
                GraphFieldState::Hidden => panic!("Скрытые поля не должны быть выделены"),
                // Если не открыто никакое поле -> режим редактирования
                _ => self.on_key_event_graph_edit(event),
            }
        } else {
            self.on_key_event_graph_default(event);
        }
    }

    /// Обрабатывает все события, связанные с нажатием клавиш в обычном режиме
    fn on_key_event_graph_default(&mut self, event: KeyEvent) {
        // Получаем состояние вкладки графика
        let state = self.graph_state_mut();

        match event.code {
            // Выход из приложения
            KeyCode::Char('q') if event.modifiers == KeyModifiers::CONTROL => self.running = false,
            // Управление вкладками
            KeyCode::BackTab => self.tabs.prev(),
            KeyCode::Tab => self.tabs.next(),
            KeyCode::Char('N') => self.open_new_tab(),
            KeyCode::Char('q') => self.tabs.close(),
            // Открытие режима редактирования (первое поле всегда не пустое)
            KeyCode::Char('e') => state.selected = Some(0),

            _ => (),
        }
    }

    /// Обрабатывает все события, связанные с нажатием клавиш в режиме редактирования
    fn on_key_event_graph_edit(&mut self, event: KeyEvent) {
        // Получаем состояние вкладки графика
        let state = self.graph_state_mut();

        match event.code {
            // Выход из режима редактирования
            KeyCode::Esc | KeyCode::Char('q') => state.selected = None,
            // Управление графиками
            KeyCode::Char('a') => self.add_graph(),
            KeyCode::Char('d') => self.remove_graph(),
            // Переключение между возможными полями
            KeyCode::BackTab => state.select_prev(1),
            KeyCode::Tab => state.select_next(1),
            KeyCode::Up => state.select_prev(4),
            KeyCode::Down => state.select_next(4),
            KeyCode::Left => state.select_prev(1),
            KeyCode::Right => state.select_next(1),
            // Открытие редактирование поля
            KeyCode::Enter => match state.selected_field_state_mut() {
                GraphFieldState::Input(input_state) => input_state.open(),
                GraphFieldState::Menu(menu_state) => menu_state.open(),
                GraphFieldState::Hidden => panic!("Скрытое поле не должно быть выделено"),
            },

            _ => (),
        }
    }

    /// Обрабатывает все события, связанные с нажатием клавиш в режиме редактирования текста
    fn on_key_event_graph_input(&mut self, event: KeyEvent) {
        // Получаем состояние открытого поля ввода
        let state = self.graph_state_mut().selected_input_state_mut();

        match event.code {
            // Выход из поля ввода
            KeyCode::Esc | KeyCode::Enter => {
                state.close();
                // Обновляем поля графиков
                self.update_graph_field(self.graph_state().selected.unwrap());
            }
            // Навигация в поле ввода
            KeyCode::Home => state.goto_start(),
            KeyCode::End => state.goto_end(),
            KeyCode::Left => state.goto_prev(),
            KeyCode::Right => state.goto_next(),
            // Ввод в поле
            KeyCode::Char(ch) => state.insert(ch),
            KeyCode::Backspace => state.remove(),

            _ => (),
        }
    }

    /// Обрабатывает все события, связанные с нажатием клавиш в режиме редактирования меню
    fn on_key_event_graph_menu(&mut self, event: KeyEvent) {
        // Получаем состояние вкладки графика
        let state = self.graph_state_mut();

        // Количество элементов в открытом меню
        let length = if state.selected.unwrap() == 0 {
            state.x_data_fields.len()
        } else {
            state.y_data_fields.len()
        };

        // Получаем состояние открытого меню
        let menu_state = state.selected_menu_state_mut();

        match event.code {
            // Выход из меню без сохранения
            KeyCode::Esc | KeyCode::Char('q') => menu_state.close(),
            // Переключение вариантов
            KeyCode::Up => menu_state.prev(length),
            KeyCode::Down => menu_state.next(length),
            // Выбор элемента в меню
            KeyCode::Enter => {
                menu_state.select();
                // Обновляем поля графиков
                self.update_graph_field(self.graph_state().selected.unwrap());
                eprintln!();
            }

            _ => (),
        }
    }

    /// Добавляет новую функцию Y(x) на график
    pub fn add_graph(&mut self) {
        // Добавляем пустые параметры к графику
        self.graph_state_mut()
            .ys_states
            .push(GraphState::default_graph());
    }

    /// Удаляет последний график
    pub fn remove_graph(&mut self) {
        // Получаем состояние вкладки графика
        let state = self.graph_state_mut();

        // Удаляем последний график, если он не последний
        if state.ys_states.len() > 1 {
            state.ys_states.pop();
        }
    }

    /// Обрабатывает поля графика
    pub fn update_graph_field(&mut self, selected: usize) {
        eprintln!("Updating {selected}");
        // Получаем состояние вкладки графика
        let state = self.graph_state_mut();

        // Обрабатываем изменение поля, откладывая апдейты некоторых полей
        let mut to_update = vec![];
        match selected {
            // Если обновилось поле данных X
            0 => {
                // Получаем состояние поля
                let field_state = state.x_states[0].menu().unwrap();
                // Получаем значение поля
                let value = &state.x_data_fields[field_state.selected().unwrap()];
                // Проверяем, является ли это полем датчика
                if let Some((sensor, _)) = value.split_once('/') {
                    // Включаем поле серийника, заодно сбрасываем
                    state.x_states[1] = GraphFieldState::new_menu();
                    to_update.push(1);

                    // Если является полем датчика, проверяем, что все графики тоже используют поле этого тачика
                    for (i, y_fields) in state.ys_states.iter_mut().enumerate() {
                        // Получаем состояние поля
                        let y_sensor_field_state = y_fields[0].menu_mut().unwrap();
                        // Если в поле что-то выбрано, отсматриваем что
                        if let Some(selection_index) = y_sensor_field_state.selected() {
                            // Если поле не начинается на название сенсора, сбрасываем значение
                            let value = &state.y_data_fields[selection_index];
                            if !value.starts_with(sensor) {
                                // Обновляем поле графика (чуть позже оно будет сброшено)
                                to_update.push((i + 1) * 4);
                            } else {
                                // Необходимо убрать серийник у поля, если уж не обновляем поле
                                y_fields[1] = GraphFieldState::Hidden;
                            }
                        }
                    }
                } else {
                    // Выключаем поле серийника
                    state.x_states[1] = GraphFieldState::Hidden;
                    to_update.push(1);

                    // Включаем поля серийника на всех установленных Y
                    for y_fields in state.ys_states.iter_mut() {
                        if y_fields[0].menu().unwrap().selected().is_some() {
                            // Устанавливаем поле. Обновлять его не нужно
                            y_fields[1] = GraphFieldState::new_menu();
                        }
                    }
                }

                // Также мы должны обновить поля мин/макс значения
                state.x_states[2] = GraphFieldState::new_input();
                state.x_states[3] = GraphFieldState::new_input();
                to_update.extend([2, 3]);

                // И под конец можно обновить перечень полей данных Y
                state.update_y_data_fields();
            }

            // Обновился серийник X
            1 => {}

            // Обновилось минимальное значение X
            2 => {}

            // Обновилось максимальное значение X
            3 => {}

            // Обновилось поле данных Y
            v if v % 4 == 0 => {
                // Получаем индекс графика
                let y_index = v / 4 - 1;
                // Получаем изменяемую ссылку на этот график
                let y_fields = &mut state.ys_states[y_index];
                // Получаем состояние поля этого графика
                let field_state = y_fields[0].menu().unwrap();
                // Проверяем, выбрано ли что-то в поле
                if let Some(selection_index) = field_state.selected() {
                    // Получаем значение поля
                    let value = &state.y_data_fields[selection_index];
                    // Получаем название датчика и поле
                    let (_, field) = value.split_once('/').unwrap();

                    // Проверяем, нужно ли отобразить серийник. Для этого смотрим, есть ли серийник у X
                    to_update.push(v + 1);
                    if let GraphFieldState::Hidden = state.x_states[1] {
                        // Ставим второе поле серийником
                        y_fields[1] = GraphFieldState::new_menu();
                    } else {
                        // Ставим второе поле пустым
                        y_fields[1] = GraphFieldState::Hidden;
                    }

                    // Проверяем на "особые поля"
                    match field {
                        "Эффективная температура" => {
                            // Для эффективной температуры, последние два поля - датчики
                            y_fields[2] = GraphFieldState::new_menu();
                            y_fields[3] = GraphFieldState::new_menu();
                            to_update.extend([v + 2, v + 3]);
                        }
                        _ => {
                            // Для всех других полей последние два поля - пустые
                            y_fields[2] = GraphFieldState::Hidden;
                            y_fields[3] = GraphFieldState::Hidden;
                            to_update.extend([v + 2, v + 3]);
                        }
                    }
                } else {
                    // Если не выбрано, скрываем все возможные поля
                    y_fields[1] = GraphFieldState::Hidden;
                    y_fields[2] = GraphFieldState::Hidden;
                    y_fields[3] = GraphFieldState::Hidden;
                    to_update.extend([v + 1, v + 2, v + 3]);
                }
            }

            // Обновился серийник Y
            v if v % 4 == 1 => {}

            // Обновился температурный датчик
            v if v % 4 == 2 => {}

            // Обновился датчик давления
            _v => {}
        }

        // Обновляем отложенное
        for i in to_update {
            self.update_graph_field(i);
        }
    }
}
