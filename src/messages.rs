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
// Встроенные функции
// ============================================================

pub fn bi_arity(name: &str, expected: usize, got: usize) -> String {
    format!("{}() ожидает {} аргумент(а/ов), передано {}", name, expected, got)
}

pub fn bi_len_bad_type(got: &str) -> String {
    format!("len() не применим к {}", got)
}

pub fn bi_num_bad_type(got: &str) -> String {
    format!("num() не применим к {}", got)
}

pub fn bi_num_bad_string(s: &str) -> String {
    format!("num(): '{}' не является числом", s)
}

pub fn bi_unknown(name: &str) -> String {
    format!("Неизвестная встроенная функция: {}", name)
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