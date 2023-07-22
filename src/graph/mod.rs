use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::state::App;

use self::state::GraphFieldState;

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
            // Открытие режима редактирования
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
            KeyCode::Enter => match state.selected_field_state() {
                GraphFieldState::Input(_) => state.selected_input_state_mut().open(),
                GraphFieldState::Menu(_) => state.selected_menu_state_mut().open(),
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
            KeyCode::Esc | KeyCode::Enter => state.close(),
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
        let length = state.sensor_fields.len();

        // Получаем состояние открытого меню
        let menu_state = state.selected_menu_state_mut();

        match event.code {
            // Выход из меню без сохранения
            KeyCode::Esc | KeyCode::Char('q') => menu_state.close(),
            // Переключение вариантов
            KeyCode::Up => menu_state.prev(length),
            KeyCode::Down => menu_state.next(length),
            // Выбор элемента в меню
            KeyCode::Enter => menu_state.select(),

            _ => (),
        }
    }

    /// Добавляет новую функцию Y(x) на график
    pub fn add_graph(&mut self) {
        // Добавляем пустые параметры к графику
        self.graph_state_mut().ys_states.push(Default::default());
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
}
