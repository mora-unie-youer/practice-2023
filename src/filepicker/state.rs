use std::{path::PathBuf, thread::JoinHandle};

/// Определяет одну элемент директории
#[derive(Clone, Debug)]
pub enum FilePickerItem {
    File(String),
    Directory(String),
}

/// Описывает данное состояние окна выбора файла
#[derive(Debug)]
pub struct FilePickerState {
    /// Сохраняет ту директорию, в которой мы сейчас находимся
    pub current_directory: PathBuf,

    /// Сохраняет содержимое директории, в которой мы находимся
    pub directory_contents: Vec<FilePickerItem>,

    /// Сохраняет выбранный в данный момент элемент директории
    pub selection_index: usize,

    /// Сохраняет отступ в списке файлов
    offset: usize,

    /// Сохраняет потоки импорта данных
    pub import_threads: Vec<JoinHandle<Result<(), ()>>>,
}

impl FilePickerState {
    /// Создаёт новое состояние выбора файла
    pub fn new() -> std::io::Result<Self> {
        // Получаем директорию, в которой мы находимся
        let current_directory = std::env::current_dir().unwrap();

        // Создаём экземпляр состояния выбора файла
        let mut state = Self {
            current_directory,
            directory_contents: Vec::new(),
            selection_index: 0,
            offset: 0,
            import_threads: Vec::new(),
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
    pub fn open_file_or_directory(&mut self) -> Option<PathBuf> {
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
    pub fn goto_parent_directory(&mut self) {
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
    pub fn goto_child_directory(&mut self, child: String) {
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
    pub fn prev_file(&mut self) {
        let length = self.directory_contents.len();
        if length == 0 {
            return;
        }

        self.selection_index = (self.selection_index + length - 1) % length;
    }

    /// Выбирает следующий файл в списке
    pub fn next_file(&mut self) {
        let length = self.directory_contents.len();
        if length == 0 {
            return;
        }

        self.selection_index = (self.selection_index + 1) % length;
    }

    /// Возвращает границы списка файлов, которые необходимо рендерить
    pub fn get_render_bounds(&mut self, max_height: usize) -> (usize, usize) {
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
