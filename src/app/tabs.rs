use super::state::AppState;

/// Структура определяющая состояние вкладок
/// Используется для определения того, в какой вкладке мы находимся, и что должны отображать
#[derive(Debug)]
pub struct Tabs<'a> {
    /// Содержит состояния вкладок
    states: Vec<AppState<'a>>,

    /// Содержит индекс активной вкладки
    pub current: usize,
}

impl Default for Tabs<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Tabs<'a> {
    /// Создаёт новое состояние вкладок
    pub fn new() -> Self {
        Self {
            // Первая вкладка всегда данные и не может быть закрыта
            states: Vec::new(),
            current: 0,
        }
    }

    /// Открывает новую вкладку
    pub fn open(&mut self, state: AppState<'a>) {
        // Добавляем новое состояние
        self.states.push(state);

        // Переключаемся на последнюю вкладку
        self.current = self.states.len() - 1;
    }

    /// Закрывает данную вкладку
    pub fn close(&mut self) {
        // Если осталась единственная вкладка - не закрываем
        if self.states.len() == 1 {
            return;
        }

        // Удаляем вкладку на данной позиции
        self.states.remove(self.current);

        // Следим за границами активной вкладки
        self.current = self.current.min(self.states.len() - 1);
    }

    /// Переключает на следующую вкладку
    pub fn next(&mut self) {
        self.current = (self.current + 1) % self.states.len();
    }

    /// Переключает на предыдущую вкладку
    pub fn prev(&mut self) {
        self.current = (self.current + self.states.len() - 1) % self.states.len();
    }

    /// Возвращает ссылку на активное состояние
    pub fn state(&self) -> &AppState<'a> {
        &self.states[self.current]
    }

    /// Возвращает изменяемую ссылку на активное состояние
    pub fn state_mut(&mut self) -> &mut AppState<'a> {
        &mut self.states[self.current]
    }

    /// Возвращает заголовки вкладок
    pub fn titles(&self) -> Vec<String> {
        self.states
            .iter()
            .enumerate()
            .map(|(i, state)| match state {
                AppState::Sensors(_) => "Датчики".to_owned(),
                AppState::Graph(_) => format!("График {i}"),
            })
            .collect()
    }
}
