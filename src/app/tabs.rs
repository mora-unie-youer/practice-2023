use crate::{graph::state::GraphState, sensors::state::SensorsState};

/// Структура определяющая состояние вкладок
/// Используется для определения того, в какой вкладке мы находимся, и что должны отображать
#[derive(Debug)]
pub struct Tabs<'a> {
    /// Содержит состояния вкладок
    states: Vec<TabState<'a>>,

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
    pub fn open(&mut self, state: TabState<'a>) {
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

    /// Обновляет поля датчиков во всех вкладках
    pub fn update_sensor_fields(&mut self) {
        for tab in &mut self.states {
            match tab {
                TabState::Graph(state) => state.update_sensor_fields(),
                TabState::Sensors(state) => state.update_sensor_fields(),
            }
        }
    }

    /// Возвращает ссылку на активное состояние
    pub fn state(&self) -> &TabState<'a> {
        &self.states[self.current]
    }

    /// Возвращает изменяемую ссылку на активное состояние
    pub fn state_mut(&mut self) -> &mut TabState<'a> {
        &mut self.states[self.current]
    }

    /// Возвращает заголовки вкладок
    pub fn titles(&self) -> Vec<String> {
        self.states
            .iter()
            .enumerate()
            .map(|(i, state)| match state {
                TabState::Sensors(_) => "Датчики".to_owned(),
                TabState::Graph(_) => format!("График {i}"),
            })
            .collect()
    }
}

/// Перечисляемый тип, определяющий режим приложения в данный момент
/// Используется для работы вкладок
#[derive(Debug)]
pub enum TabState<'a> {
    /// Вкладка с графиком
    Graph(GraphState),

    /// Вкладка с деревом датчиков
    Sensors(SensorsState<'a>),
}

impl<'a> TabState<'a> {
    /// Возвращает ссылку на состояние графика
    pub fn graph(&self) -> Option<&GraphState> {
        match self {
            Self::Graph(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает ссылку на состояние дерева сенсоров
    pub fn sensors(&self) -> Option<&SensorsState<'a>> {
        match self {
            Self::Sensors(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает изменяемую ссылку на состояние графика
    pub fn graph_mut(&mut self) -> Option<&mut GraphState> {
        match self {
            Self::Graph(state) => Some(state),
            _ => None,
        }
    }

    /// Возвращает изменяемую ссылку на состояние дерева сенсоров
    pub fn sensors_mut(&mut self) -> Option<&mut SensorsState<'a>> {
        match self {
            Self::Sensors(state) => Some(state),
            _ => None,
        }
    }
}
