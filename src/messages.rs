// ============================================================
// Лексер
// ============================================================

pub fn lex_bad_number(s: &str) -> String {
    format!("Некорректное число: {}", s)
}

pub fn lex_unclosed_string(line: usize) -> String {
    format!("Незакрытая строка (начата на строке {})", line)
}

pub fn lex_unclosed_string_generic() -> String {
    "Незакрытая строка".to_string()
}

pub fn lex_unknown_escape(seq: char) -> String {
    format!("Неизвестная escape-последовательность: \\{}", seq)
}

pub fn lex_bang_without_eq(line: usize, col: usize) -> String {
    format!("Строка {}:{}: '!' без '='. Используйте 'not'.", line, col)
}

pub fn lex_unknown_char(line: usize, col: usize, ch: char) -> String {
    format!("Строка {}:{}: неизвестный символ '{}'", line, col, ch)
}

// ============================================================
// Парсер — общие утилиты
// ============================================================

pub fn parse_expect(line: usize, col: usize, what: &str, found: &str) -> String {
    format!("Строка {}:{}: {} (встретилось: {})", line, col, what, found)
}

pub fn parse_newline_between_exprs(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: ожидался перевод строки между выражениями (встретилось: {}).",
        line, col, found
    )
}

// ============================================================
// Парсер — let / функции / параметры
// ============================================================

pub fn env_var_already_defined(name: &str) -> String {
    format!(
        "Переменная '{}' уже объявлена в этой области.\n\
         Подсказка: используйте let ^{}, если хотите изменить значение, или другую область видимости.",
        name, name
    )
}

pub fn parse_duplicate_param(line: usize, col: usize, name: &str) -> String {
    format!(
        "Строка {}:{}: параметр '{}' уже объявлен в этом списке параметров.",
        line, col, name
    )
}

pub fn parse_let_expected_name(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: после 'let' ожидалось имя переменной, встретилось {}",
        line, col, found
    )
}

pub fn parse_function_not_mutable(line: usize, col: usize) -> String {
    format!(
        "Строка {}:{}: функция не может быть изменяемой (уберите '^')",
        line, col
    )
}

pub fn parse_expected_eq_after_params() -> String {
    "ожидался '=' после параметров функции".to_string()
}

pub fn parse_expected_eq_after_name() -> String {
    "ожидался '=' после имени переменной".to_string()
}

pub fn parse_expected_eq_after_caret_name() -> String {
    "ожидался '=' после '^имя'".to_string()
}

pub fn parse_expected_lparen() -> String {
    "ожидалась '('".to_string()
}

pub fn parse_expected_param_name(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: ожидалось имя параметра, встретилось {}",
        line, col, found
    )
}

pub fn parse_expected_rparen_after_params() -> String {
    "ожидалась ')' после параметров".to_string()
}

pub fn parse_non_default_after_default(line: usize, col: usize, name: &str) -> String {
    format!(
        "Строка {}:{}: параметр '{}' без значения по умолчанию не может идти после параметра со значением по умолчанию.\n\
         Подсказка: сначала идут обязательные параметры, затем параметры со значениями по умолчанию.",
        line, col, name
    )
}

pub fn parse_star_not_last(line: usize, col: usize, name: &str) -> String {
    format!(
        "Строка {}:{}: параметр '*{}' должен быть последним в списке параметров.",
        line, col, name
    )
}

pub fn parse_star_with_default(line: usize, col: usize, name: &str) -> String {
    format!(
        "Строка {}:{}: параметр '*{}' не может иметь значения по умолчанию.",
        line, col, name
    )
}

pub fn bi_ord_empty() -> String {
    "ord(): строка пуста".to_string()
}

pub fn bi_ord_too_long(s: &str) -> String {
    format!("ord(): ожидался один символ, получено '{}'", s)
}

pub fn bi_chr_invalid(n: u32) -> String {
    format!("chr(): {} — некорректный Unicode-код", n)
}

// ============================================================
// Парсер — незакрытые конструкции (с указанием начала)
// ============================================================

pub fn parse_expected_end_named(
    construct: &str,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    found: &str,
) -> String {
    format!(
        "Строка {}:{}: ожидался 'end' для конструкции '{}', начатой в строке {}:{}, но достигнут конец (встретилось: {}).\n\
         Подсказка: у каждой '{}' должен быть парный 'end'.",
        end_line, end_col, construct, start_line, start_col, found, construct
    )
}

// ============================================================
// Парсер — echo
// ============================================================

pub fn parse_echo_no_args(line: usize, col: usize) -> String {
    format!(
        "Строка {}:{}: 'echo' без аргументов. Укажите хотя бы одно значение.",
        line, col
    )
}

// ============================================================
// Парсер — тернарник
// ============================================================

pub fn parse_ternary_expected_else() -> String {
    "в тернарном выражении ожидался 'else'".to_string()
}

pub fn parse_ternary_chain_forbidden(line: usize, col: usize) -> String {
    format!(
        "Строка {}:{}: цепочка тернарных выражений без скобок запрещена.\n\
         Подсказка: используйте скобки: a if c1 else (b if c2 else d), или if/elseif/else.",
        line, col
    )
}

// ============================================================
// Парсер — вызовы, поля, индексы
// ============================================================

pub fn parse_expected_rparen_after_args() -> String {
    "ожидалась ')' после аргументов вызова".to_string()
}

pub fn parse_expected_rbracket_after_index() -> String {
    "ожидалась ']' после индекса".to_string()
}

pub fn parse_expected_field_name(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: после '.' ожидалось имя поля, встретилось {}",
        line, col, found
    )
}

// ============================================================
// Парсер — выражения / первичные
// ============================================================

pub fn parse_expected_expr(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: ожидалось выражение, встретилось {}",
        line, col, found
    )
}

pub fn parse_interp_unclosed(line: usize, col: usize) -> String {
    format!("Строка {}:{}: незакрытая $(...)", line, col)
}

pub fn parse_interp_lex_error(e: &str) -> String {
    format!("Внутри $(...): {}", e)
}

pub fn parse_interp_trailing_tokens() -> String {
    "Внутри $(...) лишние токены после выражения".to_string()
}

// ============================================================
// Парсер — кортеж / struct / if / for / loop / lambda / match
// ============================================================

pub fn parse_expected_rparen_after_tuple() -> String {
    "ожидалась ')' после кортежа".to_string()
}

pub fn parse_expected_rparen() -> String {
    "ожидалась ')'".to_string()
}

pub fn parse_expected_rbrace() -> String {
    "ожидалась '}'".to_string()
}

pub fn parse_expected_if() -> String {
    "ожидался 'if'".to_string()
}

pub fn parse_expected_then_after_if() -> String {
    "ожидался 'then' после условия if".to_string()
}

pub fn parse_expected_then_after_elseif() -> String {
    "ожидался 'then' после условия elseif".to_string()
}

pub fn parse_expected_for() -> String {
    "ожидался 'for'".to_string()
}

pub fn parse_expected_loop_var_name(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: ожидалось имя переменной цикла, встретилось {}",
        line, col, found
    )
}

pub fn parse_expected_comma_in_for() -> String {
    "ожидалась ',' в числовом for".to_string()
}

pub fn parse_expected_do_in_for() -> String {
    "ожидался 'do' в for".to_string()
}

pub fn parse_expected_loop() -> String {
    "ожидался 'loop'".to_string()
}

pub fn parse_expected_backslash() -> String {
    "ожидался '\\'".to_string()
}

pub fn parse_expected_eq_after_lambda_params() -> String {
    "ожидался '=' после параметров лямбды".to_string()
}

pub fn parse_expected_match() -> String {
    "ожидался 'match'".to_string()
}

pub fn parse_expected_arrow_in_match() -> String {
    "ожидался '->' в ветке match".to_string()
}

pub fn parse_expected_pattern(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: ожидался паттерн, встретилось {}",
        line, col, found
    )
}

pub fn parse_expected_rparen_in_pattern() -> String {
    "ожидалась ')' в паттерне-кортеже".to_string()
}

pub fn parse_expected_eq_in_for() -> String {
    "ожидался '=' после переменной цикла".to_string()
}

pub fn parse_assign_without_caret(line: usize, col: usize) -> String {
    format!(
        "Строка {}:{}: присваивание требует '^' перед именем переменной.\n\
         Подсказка: используйте ^x = ... для изменяемой переменной или let ^x = ... при объявлении.",
        line, col
    )
}

// ============================================================
// Парсер — use
// ============================================================

pub fn parse_use_only_top_level(line: usize, col: usize) -> String {
    format!(
        "Строка {}:{}: 'use' допустим только на верхнем уровне файла, вне функций и блоков.",
        line, col
    )
}

pub fn parse_use_requires_names(module: &str) -> String {
    format!(
        "use {} без скобок и без 'as'.\n\
         Подсказка: используйте use {} (*), use {} (имя, ...) или use {} as псевдоним.",
        module, module, module, module
    )
}

pub fn parse_use_expected_lparen() -> String {
    "ожидалась '(' после имени модуля в use".to_string()
}

pub fn parse_use_expected_rparen() -> String {
    "ожидалась ')' после списка имён в use".to_string()
}

pub fn parse_use_expected_name(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: в use ожидалось имя функции, встретилось {}",
        line, col, found
    )
}

pub fn parse_use_expected_module(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: после 'use' ожидалось имя библиотеки, встретилось {}",
        line, col, found
    )
}

// ============================================================
// Интерпретатор — use
// ============================================================

pub fn use_unknown_module(name: &str) -> String {
    format!(
        "'{}' — неизвестная библиотека.\n\
         Доступные: string, math, iter, struct, tuple, file, datetime.",
        name
    )
}

pub fn use_name_conflict(name: &str) -> String {
    format!(
        "Имя '{}' уже занято в этой области.\n\
         Подсказка: используйте use ... as псевдоним (имя) или переименуйте переменную.",
        name
    )
}

pub fn use_name_not_in_module(name: &str, module: &str) -> String {
    format!("В библиотеке '{}' нет функции '{}'.", module, name)
}

// ============================================================
// Библиотеки — общие
// ============================================================

pub fn rt_user_error(msg: &str) -> String {
    msg.to_string()
}

pub fn bi_arity(name: &str, expected: usize, got: usize) -> String {
    format!("{}() ожидает {} аргумент(а/ов), передано {}", name, expected, got)
}

pub fn bi_arity_range(name: &str, min: usize, max: usize, got: usize) -> String {
    format!("{}() ожидает от {} до {} аргументов, передано {}", name, min, max, got)
}

pub fn bi_bad_type(fname: &str, expected: &str, got: &str) -> String {
    format!("{}(): ожидалось {}, получено {}", fname, expected, got)
}

pub fn bi_expected_function(fname: &str, got: &str) -> String {
    format!("{}(): ожидалась функция, получено {}", fname, got)
}

pub fn bi_index_out_of_bounds(fname: &str, i: usize, len: usize) -> String {
    format!("{}(): индекс {} вне длины {}. Индексация с 0.", fname, i, len)
}

pub fn bi_unknown(name: &str) -> String {
    format!("Неизвестная встроенная функция: {}", name)
}

// ============================================================
// Библиотека string
// ============================================================

pub fn bi_string_empty_sep() -> String {
    "string.split(): разделитель не может быть пустой строкой".to_string()
}

pub fn bi_string_count_empty_sub() -> String {
    "string.count(): подстрока не может быть пустой".to_string()
}

// ============================================================
// Библиотека math
// ============================================================

pub fn bi_math_sqrt_negative() -> String {
    "math.sqrt(): аргумент не может быть отрицательным".to_string()
}

pub fn bi_math_empty_args() -> String {
    "math.min()/math.max(): нужен хотя бы один аргумент".to_string()
}

pub fn bi_math_round_negative_digits(n: i64) -> String {
    format!("math.round(): num не может быть отрицательным, получено {}", n)
}

pub fn bi_math_log_bad_base(base: f64) -> String {
    format!("math.log(): основание должно быть > 0 и != 1, получено {}", base)
}

pub fn bi_math_log_bad_arg(x: f64) -> String {
    format!("math.log(): аргумент должен быть > 0, получено {}", x)
}

// ============================================================
// Библиотека iter
// ============================================================

pub fn bi_iter_not_array_key(key: &str) -> String {
    format!(
        "iter: struct содержит нечисловой ключ '{}'. \
         map/filter/reduce работают только с struct-массивами (все ключи — числа).\n\
         Подсказка: используйте struct.values(s) или struct.keys(s) для произвольного struct.",
        key
    )
}

// ============================================================
// Библиотека struct
// ============================================================

pub fn bi_struct_from_pairs_bad_elem(fname: &str, got: &str) -> String {
    format!("{}(): элемент должен быть кортежем из двух элементов, получено {}", fname, got)
}

pub fn bi_struct_zip_len_mismatch(fname: &str, a: usize, b: usize) -> String {
    format!("{}(): длины кортежей не совпадают ({} и {})", fname, a, b)
}

pub fn bi_struct_not_array(fname: &str) -> String {
    format!(
        "{}(): struct не является массивом (есть нечисловые ключи).\n\
         Подсказка: используйте struct.from_tuple для создания массива.",
        fname
    )
}

pub fn bi_struct_empty(fname: &str) -> String {
    format!("{}(): struct пуст", fname)
}

// ============================================================
// Присваивание полю
// ============================================================

pub fn rt_assign_field_not_struct(got: &str) -> String {
    format!("Присваивание полю применимо к struct, получено {}", got)
}

pub fn rt_assign_field_missing(path: &str) -> String {
    format!(
        "Промежуточное поле '{}' отсутствует или не является struct.\n\
         Подсказка: создайте вложенный struct сначала.",
        path
    )
}

pub fn parse_bad_assign_target(line: usize, col: usize) -> String {
    format!(
        "Строка {}:{}: слева от '=' должно быть поле или индекс (s.field = ... или s[key] = ...).\n\
         Подсказка: для присваивания переменной используйте ^name = ...",
        line, col
    )
}

pub fn parse_bad_struct_key(line: usize, col: usize) -> String {
    format!(
        "Строка {}:{}: ключ struct должен быть именем, строкой или числом.\n\
         Пример: {{ name = 1, \"key\" = 2, 0 = 3 }}",
        line, col
    )
}

// ============================================================
// Библиотека tuple
// ============================================================

pub fn bi_tuple_zip_len_mismatch(fname: &str, a: usize, b: usize) -> String {
    format!("{}(): длины кортежей не совпадают ({} и {})", fname, a, b)
}

// ============================================================
// Библиотека file
// ============================================================

pub fn bi_file_read_error(path: &str, err: &std::io::Error) -> String {
    format!("file.read(): не удалось прочитать '{}': {}", path, err)
}

pub fn bi_file_write_error(path: &str, err: &std::io::Error) -> String {
    format!("file.write(): не удалось записать '{}': {}", path, err)
}

pub fn bi_file_append_error(path: &str, err: &std::io::Error) -> String {
    format!("file.append(): не удалось дописать '{}': {}", path, err)
}

pub fn bi_file_remove_error(path: &str, err: &std::io::Error) -> String {
    format!("file.remove(): не удалось удалить '{}': {}", path, err)
}

pub fn bi_file_remove_missing(path: &str) -> String {
    format!("file.remove(): файл '{}' не существует", path)
}

pub fn bi_file_read_lines_error(path: &str, err: &std::io::Error) -> String {
    format!("file.read_lines(): не удалось прочитать '{}': {}", path, err)
}

// ============================================================
// Библиотека datetime
// ============================================================

pub fn bi_datetime_bad_timestamp() -> String {
    "datetime.from_timestamp(): некорректный timestamp".to_string()
}

pub fn bi_datetime_bad_struct(field: &str) -> String {
    format!("datetime: поле '{}' должно быть числом", field)
}

pub fn bi_datetime_mktime_bad(field: &str, got: &str) -> String {
    format!("datetime.mktime(): некорректное поле '{}': {}", field, got)
}

// ============================================================
// Окружение
// ============================================================

pub fn env_assign_immutable(name: &str) -> String {
    format!(
        "Переменная '{}' объявлена как неизменяемая (let {}).\n\
         Подсказка: используйте let ^{} при объявлении, если значение нужно менять.",
        name, name, name
    )
}

pub fn env_assign_undefined(name: &str) -> String {
    format!(
        "Переменная '{}' не объявлена.\n\
         Подсказка: используйте let ^{} = ... для создания изменяемой переменной.",
        name, name
    )
}

pub fn env_var_undefined(name: &str) -> String {
    format!(
        "Переменная '{}' не объявлена.\n\
         Подсказка: объявите её через let {} = ...",
        name, name
    )
}

// ============================================================
// Интерпретатор — управление потоком
// ============================================================

pub fn rt_break_outside_loop() -> String {
    "'break' вне цикла".to_string()
}

pub fn rt_continue_outside_loop() -> String {
    "'continue' вне цикла".to_string()
}

pub fn rt_return_in_expr() -> String {
    "'return' внутри выражения не поддерживается".to_string()
}

pub fn rt_break_in_expr() -> String {
    "'break' внутри выражения не поддерживается".to_string()
}

pub fn rt_continue_in_expr() -> String {
    "'continue' внутри выражения не поддерживается".to_string()
}

// ============================================================
// Интерпретатор — операции
// ============================================================

pub fn rt_unary_neg_type(got: &str) -> String {
    format!("Унарный '-' применим только к числу, получено {}", got)
}

pub fn rt_for_step_zero() -> String {
    "Шаг for не может быть нулём".to_string()
}

pub fn rt_div_by_zero() -> String {
    "Деление на ноль".to_string()
}

pub fn rt_mod_by_zero() -> String {
    "Остаток от деления на ноль".to_string()
}

pub fn rt_index_out_of_bounds(i: usize, len: usize) -> String {
    format!("Индекс {} вне кортежа длины {}. Индексация с 0.", i, len)
}

pub fn rt_field_missing(name: &str) -> String {
    format!("Поле '{}' отсутствует в struct", name)
}

pub fn rt_index_on_non_collection(got: &str) -> String {
    format!("Индексация применима к кортежу и struct, получено {}", got)
}

pub fn rt_field_on_non_struct(got: &str) -> String {
    format!("Доступ через '.' применим к struct, получено {}", got)
}

pub fn rt_forin_bad_iter(got: &str) -> String {
    format!("for..in ожидает кортеж или struct, получено {}", got)
}

// ============================================================
// Интерпретатор — вызовы
// ============================================================

pub fn rt_too_few_args(required: usize, got: usize, param: &str) -> String {
    format!(
        "Недостаточно аргументов при вызове функции: ожидался '{}', передано {} (нужно минимум {})",
        param, got, required
    )
}

pub fn rt_too_many_args(expected: usize, got: usize) -> String {
    format!(
        "Слишком много аргументов при вызове функции: ожидалось {}, передано {}",
        expected, got
    )
}

pub fn rt_call_non_function(got: &str) -> String {
    format!("Нельзя вызвать значение типа {}", got)
}



// ============================================================
// Типизированные контексты as_num
// ============================================================

pub fn rt_arith_bad_type(got: &str) -> String {
    format!("арифметика: ожидалось число, получено {}", got)
}

pub fn rt_compare_bad_type(got: &str) -> String {
    format!("сравнение: ожидалось число, получено {}", got)
}

pub fn rt_index_bad_type(got: &str) -> String {
    format!("индекс кортежа: ожидалось число, получено {}", got)
}

pub fn rt_for_bad_type(got: &str) -> String {
    format!("for: ожидалось число, получено {}", got)
}

// ============================================================
// CLI
// ============================================================

pub fn cli_usage() -> String {
    "Использование: simpl <файл.simpl>".to_string()
}

pub fn cli_read_error(path: &str, err: &std::io::Error) -> String {
    format!("Не удалось прочитать файл '{}': {}", path, err)
}

pub fn cli_version(version: &str) -> String {
    format!("simpl {}", version)
}

pub fn cli_help() -> String {
    "\
Использование: simpl [опции] [файл.spl]

Опции:
  -h, --help       Показать эту справку
  -v, --version    Показать версию
  -c, --check      Проверить синтаксис файла без выполнения
  -r, --repl       Интерактивная среда (пока не реализована)

Примеры:
  simpl program.spl
  simpl --check program.spl
  simpl -v
"
    .to_string()
}

pub fn cli_repl_not_implemented() -> String {
    "REPL пока не реализован. Будет добавлен в следующих версиях.".to_string()
}

pub fn cli_missing_file() -> String {
    "Не указан файл. Используйте simpl --help для справки.".to_string()
}

pub fn cli_unknown_flag(flag: &str) -> String {
    format!("Неизвестная опция '{}'. Используйте simpl --help для справки.", flag)
}

pub fn cli_too_many_files() -> String {
    "Указано больше одного файла. simpl принимает один файл.".to_string()
}

pub fn cli_check_ok(path: &str) -> String {
    format!("{}: синтаксис в порядке", path)
}

// ============================================================
// input
// ============================================================

pub fn input_prompt(prompt: &str) -> String {
    prompt.to_string()
}

pub fn input_read_error(err: &std::io::Error) -> String {
    format!("input(): не удалось прочитать строку: {}", err)
}

// ============================================================
// import
// ============================================================

pub fn parse_import_only_top_level(line: usize, col: usize) -> String {
    format!(
        "Строка {}:{}: 'import' допустим только на верхнем уровне файла, вне функций и блоков.",
        line, col
    )
}

pub fn parse_import_expected_path(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: ожидался путь файла после 'import', встретилось {}",
        line, col, found
    )
}

pub fn parse_import_expected_segment(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: в пути import ожидалось имя сегмента, встретилось {}",
        line, col, found
    )
}

pub fn import_cycle(path: &str) -> String {
    format!("Циклический импорт: {}", path)
}

pub fn import_read_error(path: &str, err: &std::io::Error) -> String {
    format!("import: не удалось прочитать '{}': {}", path, err)
}

pub fn parse_import_requires_names(path: &str) -> String {
    format!(
        "import {} без скобок и без 'as'.\n\
         Подсказка: используйте import {} (*), import {} (имя, ...) или import {} as псевдоним.",
        path, path, path, path
    )
}

// ============================================================
// REPL
// ============================================================

pub fn repl_help() -> String {
    "\
Команды:
  :q, :quit        выйти
  :h, :help        эта справка
  :reset           сбросить окружение
  :load <file>     выполнить файл в текущем окружении

Ввод:
  Многострочные конструкции (if, for, let ... (...), match) — продолжение
  вводится с приглашением '... '. Закончите 'end'.
  
  Результат последнего выражения печатается (включая nil).
"
    .to_string()
}

pub fn repl_unknown_command(cmd: &str) -> String {
    format!("Неизвестная команда: {}", cmd)
}

pub fn repl_load_usage() -> String {
    "Использование: :load <file>".to_string()
}

pub fn repl_load_error(path: &str, err: &std::io::Error) -> String {
    format!(":load: не удалось прочитать '{}': {}", path, err)
}

pub fn repl_reset_ok() -> String {
    "Окружение сброшено.".to_string()
}
// ============================================================

pub fn rt_compose_non_function(got: &str) -> String {
    format!("Композиция (..) ожидает функции, получено {}", got)
}

pub fn rt_flip_non_function(got: &str) -> String {
    format!("flip(): ожидалась функция, получено {}", got)
}

pub fn rt_flip_needs_binary(required: usize) -> String {
    format!(
        "flip(): ожидалась функция ровно с двумя обязательными параметрами, найдено {}.\n\
         Подсказка: flip работает только с бинарными функциями.",
        required
    )
}

pub fn rt_flip_builtin() -> String {
    "flip(): встроенные функции не поддерживаются.\n\
     Подсказка: оберните встроенную функцию в лямбду: flip(\\(a, b) = ... end)".to_string()
}

// ============================================================
// Struct comprehension
// ============================================================

pub fn parse_comp_expected_for(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: после ':' в comprehension ожидался 'for', встретилось {}.\n\
         Подсказка: {{ key: value for var in iterable }} или {{ key: value for k, v in iterable }}.",
        line, col, found
    )
}

pub fn parse_comp_expected_var(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: в comprehension ожидалось имя переменной, встретилось {}",
        line, col, found
    )
}

pub fn parse_comp_expected_in(line: usize, col: usize, found: &str) -> String {
    format!(
        "Строка {}:{}: в comprehension ожидался 'in', встретилось {}",
        line, col, found
    )
}

// ============================================================
// Побитовые операции
// ============================================================

pub fn rt_bitop_expected_int(op: &str, got: &str) -> String {
    format!(
        "{}: ожидалось целое число, получено {}",
        op, got
    )
}

pub fn rt_bitop_overflow(op: &str, n: f64) -> String {
    format!(
        "{}: число {} вне диапазона целых (|n| < 2^53)",
        op, n
    )
}

pub fn rt_shift_too_big(op: &str, n: i64) -> String {
    format!(
        "{}: сдвиг на {} недопустим (0 <= n < 64)",
        op, n
    )
}

// ============================================================
// Checker (статические ошибки)
// ============================================================

pub fn check_var_undefined(name: &str) -> String {
    format!(
        "Переменная '{}' не объявлена.\n\
         Подсказка: объявите её через let {} = ...",
        name, name
    )
}

pub fn check_assign_undefined(name: &str) -> String {
    format!(
        "Переменная '{}' не объявлена.\n\
         Подсказка: используйте let ^{} = ... для создания изменяемой переменной.",
        name, name
    )
}

pub fn check_assign_immutable(name: &str) -> String {
    format!(
        "Переменная '{}' объявлена как неизменяемая (let {}).\n\
         Подсказка: используйте let ^{} при объявлении, если значение нужно менять.",
        name, name, name
    )
}

pub fn check_var_already_defined(name: &str) -> String {
    format!(
        "Переменная '{}' уже объявлена в этой области.\n\
         Подсказка: используйте let ^{}, если хотите изменить значение, или другую область видимости.",
        name, name
    )
}

pub fn check_break_outside_loop() -> String {
    "'break' вне цикла".to_string()
}

pub fn check_continue_outside_loop() -> String {
    "'continue' вне цикла".to_string()
}

pub fn check_use_unknown_module(name: &str) -> String {
    format!(
        "'{}' — неизвестная библиотека.\n\
         Доступные: string, math, iter, struct, tuple, file, datetime.",
        name
    )
}

pub fn check_use_name_not_in_module(name: &str, module: &str) -> String {
    format!("В библиотеке '{}' нет функции '{}'.", module, name)
}

pub fn check_import_cycle(path: &str) -> String {
    format!("Циклический импорт: {}", path)
}

pub fn check_flip_builtin() -> String {
    "flip(): встроенные функции не поддерживаются.\n\
     Подсказка: оберните встроенную функцию в лямбду: flip(\\(a, b) = ... end)".to_string()
}

pub fn check_flip_needs_binary(required: usize) -> String {
    format!(
        "flip(): ожидалась функция ровно с двумя обязательными параметрами, найдено {}.",
        required
    )
}

pub fn check_flip_non_function(got: &str) -> String {
    format!("flip(): ожидалась функция, получено {}", got)
}

pub fn check_compose_non_function(got: &str) -> String {
    format!("Композиция (..) ожидает функции, получено {}", got)
}

pub fn check_too_many_args(expected: usize, got: usize) -> String {
    format!(
        "Слишком много аргументов при вызове функции: ожидалось {}, передано {}",
        expected, got
    )
}

pub fn check_bi_arity(name: &str, expected: usize, got: usize) -> String {
    format!("{}() ожидает {} аргумент(а/ов), передано {}", name, expected, got)
}

pub fn check_bi_unknown(name: &str) -> String {
    format!("Неизвестная функция: {}", name)
}

pub fn check_call_non_function(got: &str) -> String {
    format!("Нельзя вызвать значение типа {}", got)
}

pub fn check_for_step_zero() -> String {
    "Шаг for не может быть нулём".to_string()
}

pub fn check_index_out_of_bounds(i: usize, len: usize) -> String {
    format!("Индекс {} вне кортежа длины {}. Индексация с 0.", i, len)
}