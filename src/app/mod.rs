use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyEventKind};
use tui::{backend::Backend, Terminal};

use self::state::App;

pub mod state;
pub mod tabs;

/// Функция для запуска приложения в данном терминале
pub fn run_application<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> std::io::Result<()> {
    const TICK_RATE: Duration = Duration::from_millis(250);

    // Время, когда последний тик был выполнен
    let mut last_tick = Instant::now();
    loop {
        // Проверяем, завершило ли приложение работу
        if !app.running {
            break Ok(());
        }

        // Рисуем интерфейс в данном нам фрейме
        terminal.draw(|frame| crate::ui::draw(frame, app))?;

        // Проверяем, прошёл ли один тик
        if last_tick.elapsed() >= TICK_RATE {
            last_tick = Instant::now();
            app.tick();
        }

        // Проверяем инпут с клавиатуры
        let event_timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));
        if crossterm::event::poll(event_timeout)? {
            // Проверяем вид инпута
            match crossterm::event::read()? {
                // Если это ввод с клавиатуры - обрабатываем
                // Обрабатываем всё кроме Release (спасибо винда за двойные нажатия)
                Event::Key(event) if event.kind != KeyEventKind::Release => {
                    app.on_key_event(event)?
                }

                // Если это ввод с мыши - обрабатываем
                Event::Mouse(event) => app.on_mouse_event(event)?,
                // Все остальные инпуты меня пока не интересуют
                _ => (),
            }
        }
    }
}
