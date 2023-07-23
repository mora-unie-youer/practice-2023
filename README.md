# Практика МФ МГТУ им. Н.Э. Баумана на 2023 год
Представляет из себя приложения для анализа и отображения данных с датчиков приборов МФ МГТУ им. Н.Э. Баумана.
Является консольным приложением.

## Использованные библиотеки (крейты)
- `itertools` - крейт для удобных функций итераторов
- `chrono` - используется для чтения дат в файлах данных
- `serde` и `serde_json` - используются для парсинга файлов данных
- `rusqlite` - используется для взаимодействия с БД SQLite
- `crossterm` и `tui-rs` - используются для TUI интерфейса

## Кроссплатформенность
Судя по проведённым мною тестам, приложение должно работать на всех популярных ОС, а именно
Windows и Unix-like. Если у вас на микроволновке не запустилось, эта ваша проблема. Однако учитывайте,
что создатель (т.е. я) создавал на ОС Linux, потому на других ОС вполне возможны баги.

## Инструкция по сборке
Очевидно, необходим компилятор ЯП `Rust`. Желательно установить последнюю Stable версию.
(Слава богу тут я не использовал Nightly фич компилятора).

Далее необходимо собрать бинарник: `cargo build --release`.
И бинарник будет ждать вас по пути: `./target/release/practice`.

## Инструкция по использованию
### Сочетания клавиш
Приложение представляет из себя консольное приложение с вкладками, и в каждой вкладке, а также её режиме,
вы можете наблюдать разное управление. Управление осуществляется посредством клавиатуры.
Также учитывайте, что ниже представлены сочетания клавиш в особом регистре и нотации:
- `n` - без каких либо модификаторов
- `N` или `S-n` - Shift+n
- `C-n` - Ctrl+n
- `C-N` или `C-S-n` - Ctrl+Shift+n

Управление во всех вкладках (обычный режим вкладок):
- `C-q` - выход из приложения
- `Tab` - переход к следующей вкладке
- `S-Tab` - переход к предыдущей вкладке
- `N` - открытие новой вкладки "График"

#### Вкладка "Дерево сенсоров"
Управление в обычном режиме:
- `q` - выход из приложения
- `Up`, `Down`, `Left`, `Right` (стрелки) - навигация по дереву сенсоров
- `Space` или `Enter` - раскрытие/скрытие пункта дерева
- `i` - открытие окна импорта файла/директории (в данной директории)

Управление в окне импорта файла/директории:
- `Esc` или `q` - закрытие окна
- `Up`, `Down` - переключение между файлами вверх/вниз
- `Left` - переход в родительскую директорию, если этого возможно
- `Right` - открывает файл/переходит в директорию-потомка
- `I` - импортирует все файлы в данной директории (**не выделенной, а данной**)

#### Вкладка "График"
Управление в обычном режиме:
- `q` - закрывает данную вкладку
- `e` - включает режим редактирования полей
**Примечание**: график отрисовывается только в обычном режиме.

Управление в режиме редактирования полей:
- `q` - выход из режима редактирования полей
- `a` - добавляет новый график
- `d` - удаляет последний график
- `S-Tab` или `Left` - переходит к предыдущему полю
- `Tab` или `Right` - переходит к следующему полю
- `Up`, `Down` - переходит на строчку вверх/вниз
- `Enter` - открывает редактирование выбранного поля
**Примечание**: при выходе из режима редактирования может быть небольшое зависание программы,
т.к. обрабатываются данные для отображения на графике.
**Примечание 2**: если после редактирования появились ошибки в полях "ввода текста", ничего не обновится.

Управление в режиме редактирования поля "Ввод текста":
- `Esc` или `Enter` - сохраняет поле
- `Left`, `Right` - перемещает курсор влево/вправо
- `Home`, `End` - перемещает курсор в начало/конец поля ввода
- `Backspace` - стирает символ перед курсором
- Любой символ подлежит вводу в поле

Управление в режиме редактирования поля "Меню":
- `Esc` или `q` - выходит из меню без сохранения выбора
- `Enter` - выходит из меню с сохранением выбора
- `Up`, `Down` - переключается между пунктами меню вверх/вниз
