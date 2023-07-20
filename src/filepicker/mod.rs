use crossterm::event::{KeyCode, KeyEvent};

use crate::app::state::{App, AppState};

use self::state::{FilePickerItem, FilePickerState};

pub mod state;
pub mod ui;

impl App<'_> {
    /// Открывает окно выбора файла
    pub fn open_filepicker(&mut self) -> std::io::Result<()> {
        // Подготавливаем состояние выбора файла
        let state = FilePickerState::new()?;
        // Устанавливаем новое состояние
        self.state = AppState::FilePicker(state);

        Ok(())
    }

    /// Закрывает окно выбора файла
    fn close_filepicker(&mut self) {
        self.state = AppState::Default;
    }

    /// Выполняет один тик в режиме выбора файла
    pub fn tick_filepicker(&mut self) {
        // Получаем состояние выбора файла
        let state = self.state.file_picker_state().unwrap();

        // Если происходит процесс импорта
        if !state.import_threads.is_empty() {
            // Ждём, пока все потоки выполнятся...
            while let Some(thread) = state.import_threads.pop() {
                // Ждём отдельный поток
                let _ = thread.join().unwrap();
            }

            // После того, как дождались - можем закрыть выбор файла
            self.close_filepicker();

            // Также мы должны обновить дерево датчиков
            // TODO: на всякий случай нужна обработка ошибок здесь
            let _ = self.update_sensors_tree();
        }
    }

    /// Обрабатывает все события, связанные с нажатием клавиш в режиме выбора файла
    pub fn on_key_event_filepicker(&mut self, event: KeyEvent) -> std::io::Result<()> {
        // Получаем состояние, для того чтобы поменять что-нибудь
        let state = self.state.file_picker_state().unwrap();

        match event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.close_filepicker(),
            KeyCode::Up => state.prev_file(),
            KeyCode::Down => state.next_file(),
            KeyCode::Left => state.goto_parent_directory(),
            KeyCode::Right => self.try_import_file(),
            KeyCode::Char('I') => self.try_import_directory(),
            _ => (),
        }

        Ok(())
    }

    /// Пытается открыть файл, и если он был открыт -> импортирует данные в БД
    fn try_import_file(&mut self) {
        // Получаем состояние, для того чтобы открыть файл/директорию
        let state = self.state.file_picker_state().unwrap();

        // Пытаемся открыть файл. Если директория, то выходим из функции
        let file_path = match state.open_file_or_directory() {
            Some(path) => path,
            None => return,
        };

        // Импортируем. Если имеем ошибку, переходим к следующему
        let thread = self.import_json_file_to_database(file_path);

        // Добавляем поток импорта в список для ожидания
        let state = self.state.file_picker_state().unwrap();
        state.import_threads.push(thread);
    }

    /// Пытается импортировать данные из всех файлов в данной директории
    fn try_import_directory(&mut self) {
        // Получаем состояние, для того чтобы открыть файл/директорию
        let state = self.state.file_picker_state().unwrap();

        // Получаем директорию, в которой мы находимся
        let current_directory = state.current_directory.clone();

        // Читаем файлы из директории и импортируем их
        let items = state.directory_contents.clone();
        for item in items {
            // Если элемент - директория, пропускаем. Иначе -> импортируем
            match item {
                FilePickerItem::Directory(_) => continue,
                FilePickerItem::File(filename) => {
                    // Собираем путь до файла
                    let file_path = current_directory.join(filename);

                    // Импортируем. Если имеем ошибку, переходим к следующему
                    let thread = self.import_json_file_to_database(file_path);

                    // Добавляем поток импорта в список для ожидания
                    let state = self.state.file_picker_state().unwrap();
                    state.import_threads.push(thread);
                }
            }
        }
    }
}
