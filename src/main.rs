use std::time::{Duration, Instant};

use app::App;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

pub mod app;
pub mod database;
pub mod filepicker;
pub mod sensors_tree;
pub mod ui;
pub mod utils;

/// Функция для запуска приложения в данном терминале
fn run_application<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> std::io::Result<()> {
    const TICK_RATE: Duration = Duration::from_millis(250);

    // Время, когда последний тик был выполнен
    let mut last_tick = Instant::now();
    loop {
        // Проверяем, завершило ли приложение работу
        if !app.running {
            break Ok(());
        }

        // Рисуем интерфейс в данном нам фрейме
        terminal.draw(|frame| ui::draw(frame, app))?;

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
                Event::Key(event) => app.on_key_event(event)?,
                // Если это ввод с мыши - обрабатываем
                Event::Mouse(event) => app.on_mouse_event(event)?,
                // Все остальные инпуты меня пока не интересуют
                _ => (),
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Запускаем SQLite базу данных
    let database = rusqlite::Connection::open("db.sqlite")?;

    // Получаем stdout для манипуляций с интерфесом
    let mut stdout = std::io::stdout();

    // Подготавливаем терминал к графическому интерфейсу
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Запускаем приложение с интерфейсом
    let mut app = App::new(database)?;
    let result = run_application(&mut terminal, &mut app);

    // Восстанавливаем терминал до рабочего состояния
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Если была глобальная ошибка, мы теперь можем её вывести
    if let Err(err) = result {
        eprintln!("An error occurred: {err:?}");
        std::process::exit(1);
    }

    Ok(())
}
