use tui::{backend::Backend, Frame};

use crate::app::App;

/// Основная функция рендера интерфейса
pub fn draw<B: Backend>(frame: &mut Frame<B>, app: &mut App) {}
