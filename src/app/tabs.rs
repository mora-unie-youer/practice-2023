/// Структура определяющая состояние вкладок
/// Используется для определения того, в какой вкладке мы находимся, и что должны отображать
#[derive(Debug)]
pub struct Tabs {
    /// Содержит заголовки вкладок
    pub titles: Vec<String>,

    /// Содержит индекс активной вкладки
    pub current: usize,
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Tabs {
    /// Создаёт новое состояние вкладок
    pub fn new() -> Self {
        Self {
            // Первая вкладка всегда данные и не может быть закрыта
            titles: vec!["Данные".into(), "График 1".into(), "График 2".into()],
            current: 0,
        }
    }

    /// Переключает на следующую вкладку
    pub fn next(&mut self) {
        self.current = (self.current + 1) % self.titles.len();
    }

    /// Переключает на предыдущую вкладку
    pub fn prev(&mut self) {
        self.current = (self.current + self.titles.len() - 1) % self.titles.len();
    }
}
