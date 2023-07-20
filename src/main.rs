use app::{run_application, state::App};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{backend::CrosstermBackend, Terminal};

pub mod app;
pub mod database;
pub mod filepicker;
pub mod graph;
pub mod sensors;
pub mod ui;

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
