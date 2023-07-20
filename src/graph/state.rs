/// Сохраняет состояние вкладки графика
#[derive(Debug)]
pub struct GraphState {}

impl Default for GraphState {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphState {
    /// Создаёт новый экземпляр состояния вкладки графика
    pub fn new() -> Self {
        GraphState {}
    }
}
