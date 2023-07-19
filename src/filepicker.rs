use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::{App, AppState},
    utils::{get_inner_block_area, get_popup_area},
};

impl App {
    /// Открывает окно выбора файла
    pub fn open_filepicker(&mut self) -> std::io::Result<()> {
        // Подготавливаем состояние выбора файла
        let state = FilePickerState::new()?;
        // Устанавливаем новое состояние
        self.state = AppState::FilePicker(state);

        Ok(())
    }

    fn close_filepicker(&mut self) {
        self.state = AppState::Default;
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
        let _ = self.import_file_to_database(file_path);

        // Закрываем выбор файлов, т.к. мы закончили процесс
        self.close_filepicker();
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
                    let _ = self.import_file_to_database(file_path);
                }
            }
        }

        // Закрываем выбор файлов, т.к. мы закончили процесс
        self.close_filepicker();
    }
}

/// Описывает данное состояние окна выбора файла
#[derive(Debug)]
pub struct FilePickerState {
    /// Сохраняет ту директорию, в которой мы сейчас находимся
    pub current_directory: PathBuf,

    /// Сохраняет содержимое директории, в которой мы находимся
    directory_contents: Vec<FilePickerItem>,

    /// Сохраняет выбранный в данный момент элемент директории
    selection_index: usize,

    /// Сохраняет отступ в списке файлов
    offset: usize,
}

impl FilePickerState {
    /// Создаёт новое состояние выбора файла
    fn new() -> std::io::Result<Self> {
        // Получаем директорию, в которой мы находимся
        let current_directory = std::env::current_dir().unwrap();

        // Создаём экземпляр состояния выбора файла
        let mut state = Self {
            current_directory,
            directory_contents: Vec::new(),
            selection_index: 0,
            offset: 0,
        };

        // Пополняем состояние файлами и директориями
        state.populate_contents()?;

        Ok(state)
    }

    /// Пополняет состояние файлами и директориями, которые мы видим в данный момент
    fn populate_contents(&mut self) -> std::io::Result<()> {
        // Получаем элементы директории и превращаем их в необходимый тип
        let mut directory_items: Vec<_> = std::fs::read_dir(&self.current_directory)?
            .map(|file| file.unwrap().path())
            .collect();
        directory_items.sort();

        let directory_contents = directory_items
            .into_iter()
            .map(|path| {
                let metadata = std::fs::metadata(&path).unwrap();
                if metadata.is_dir() {
                    FilePickerItem::Directory(path.to_str().unwrap().to_owned())
                } else {
                    FilePickerItem::File(path.to_str().unwrap().to_owned())
                }
            })
            .collect();

        self.directory_contents = directory_contents;
        Ok(())
    }

    /// Открывает файл или переходит в директорию
    fn open_file_or_directory(&mut self) -> Option<PathBuf> {
        // Проверяем, есть ли у нас вообще файлы/директории
        if self.directory_contents.is_empty() {
            return None;
        }

        // Проверяем то, на чём у нас сейчас курсор
        match &self.directory_contents[self.selection_index] {
            FilePickerItem::File(filename) => Some(self.current_directory.join(filename)),
            FilePickerItem::Directory(dirname) => {
                self.goto_child_directory(dirname.clone());
                None
            }
        }
    }

    /// Открывает директорию-родителя
    fn goto_parent_directory(&mut self) {
        // Сохраняем директорию, на случай ошибок при переходе
        let backup = self.current_directory.clone();

        // Получаем родительскую директорию или выходим из функции
        let parent_directory = match backup.parent() {
            Some(path) => path,
            None => return,
        };

        // Переходим в эту директорию
        self.current_directory = parent_directory.to_path_buf();

        // Получаем файлы в этой директории. Если не выходит -> не открываем директорию
        if self.populate_contents().is_err() {
            self.current_directory = backup;
        } else {
            // Сбрасываем индекс выбора, чтобы не получать ошибок
            self.selection_index = 0;
        }
    }

    /// Открывает директорию-потомка
    fn goto_child_directory(&mut self, child: String) {
        // Сохраняем директорию, на случай ошибок при переходе
        let backup = self.current_directory.clone();

        // Получаем директорию-потомка
        let child_directory = self.current_directory.join(child);

        // Переходим в эту директорию
        self.current_directory = child_directory;

        // Получаем файлы в этой директории. Если не выходит -> не открываем директорию
        if self.populate_contents().is_err() {
            self.current_directory = backup;
        } else {
            // Сбрасываем индекс выбора, чтобы не получать ошибок
            self.selection_index = 0;
        }
    }

    /// Выбирает предыдущий файл в списке
    fn prev_file(&mut self) {
        let length = self.directory_contents.len();
        if length == 0 {
            return;
        }

        self.selection_index = (self.selection_index + length - 1) % length;
    }

    /// Выбирает следующий файл в списке
    fn next_file(&mut self) {
        let length = self.directory_contents.len();
        if length == 0 {
            return;
        }

        self.selection_index = (self.selection_index + 1) % length;
    }

    /// Возвращает границы списка файлов, которые необходимо рендерить
    fn get_render_bounds(&mut self, max_height: usize) -> (usize, usize) {
        // Получаем элементы для более удобного доступа
        let items = &self.directory_contents;

        // Подготавливаем отступ, начало, конец, ширину
        let offset = self.offset.min(items.len().saturating_sub(1));
        let mut start = offset;
        let mut height = max_height.min(items.len() - offset);
        let mut end = offset + height;

        // Сдвигаемся, пока мы не имеем выделение в правой границе
        while self.selection_index >= end {
            height += 1;
            end += 1;
            if height > max_height {
                height -= 1;
                start += 1;
            }
        }

        // Сдвигаемся, пока не имеем выделение в левой границе
        while self.selection_index < start {
            start -= 1;
            height += 1;
            if height > max_height {
                end -= 1;
                height -= 1;
            }
        }

        self.offset = start;
        (start, end)
    }
}

/// Определяет одну элемент директории
#[derive(Clone, Debug)]
pub enum FilePickerItem {
    File(String),
    Directory(String),
}

pub fn draw_file_picker<B: Backend>(frame: &mut Frame<B>, app: &mut App, area: Rect) {
    // Получаем состояние выбора файла
    let state = app.state.file_picker_state().unwrap();

    // Выделяем область под окошко
    let popup_area = get_popup_area(90, 80, area);

    // Делаем блок
    let block = Block::default()
        .title("Выбор файла для импорта")
        .borders(Borders::ALL);

    // Рендерим блок
    frame.render_widget(block, popup_area);

    // Рендерим список файлов
    draw_file_list(frame, state, popup_area);
}

fn draw_file_list<B: Backend>(frame: &mut Frame<B>, state: &mut FilePickerState, area: Rect) {
    // Выделяем область под список файлов
    let inner_area = get_inner_block_area(area);

    // Если директория не пустая -> обрабатываем файлы
    if !state.directory_contents.is_empty() {
        // Набор названий файлов и директорий
        let mut filenames: Vec<_> = state
            .directory_contents
            .iter()
            .map(|item| match item {
                FilePickerItem::File(f) => f.rsplit('/').next().unwrap().to_owned(),
                FilePickerItem::Directory(f) => format!("{}/", f.rsplit('/').next().unwrap()),
            })
            .map(Span::raw)
            .collect();

        // Делаем выбранный файл выделенным
        filenames[state.selection_index].style = Style::default()
            .fg(Color::Indexed(2))
            .add_modifier(Modifier::BOLD);

        // Получаем границы рендера
        let (start, end) = state.get_render_bounds(inner_area.height as usize);

        // Делаем Spans для нормального отображения
        let filenames_spans: Vec<_> = filenames
            .into_iter()
            .skip(start)
            .take(end - start)
            .map(Spans::from)
            .collect();

        // Подготавливаем виджет для отображения
        let paragraph = Paragraph::new(filenames_spans);
        frame.render_widget(paragraph, inner_area);
    } else {
        // Делаем виджет для рендера внутри
        let text = "--- Директория пуста --";
        let paragraph = Paragraph::new(Text::from(text));
        frame.render_widget(paragraph, inner_area);
    }
}
