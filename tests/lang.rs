use simpl::run_source_capture;

fn run(src: &str) -> String {
    run_source_capture(src).expect("программа должна выполниться без ошибок")
}

fn run_err(src: &str) -> String {
    run_source_capture(src).expect_err("программа должна упасть с ошибкой")
}

#[test]
fn let_immutable() {
    assert_eq!(run("let x = 1\necho. x"), "1\n");
}

#[test]
fn let_mutable_assign() {
    assert_eq!(run("let ^x = 1\n^x = 2\necho. x"), "2\n");
}

#[test]
fn assign_without_caret_error() {
    let e = run_err("let x = 1\nx = 2");
    assert!(e.contains("требует '^'"), "сообщение: {}", e);
}

#[test]
fn assign_to_immutable_with_caret_error() {
    let e = run_err("let x = 1\n^x = 2");
    assert!(e.contains("неизменяемая"), "сообщение: {}", e);
}

#[test]
fn let_undefined_assign_error() {
    let e = run_err("^y = 1");
    assert!(e.contains("не объявлена"), "сообщение: {}", e);
}

#[test]
fn function_implicit_return() {
    assert_eq!(run("let f (x) = x + 1 end\necho. f(10)"), "11\n");
}

#[test]
fn function_default_param() {
    assert_eq!(run("let f (x, y = 5) = x + y end\necho. f(10)"), "15\n");
    assert_eq!(run("let f (x, y = 5) = x + y end\necho. f(10, 1)"), "11\n");
}

#[test]
fn function_star_args() {
    assert_eq!(run("let f (*rest) = len(rest) end\necho. f(1, 2, 3)"), "3\n");
}

#[test]
fn function_default_sees_previous_param() {
    assert_eq!(run("let f (a, b = a + 1) = a + b end\necho. f(3)"), "7\n");
}

#[test]
fn function_too_many_args_error() {
    let e = run_err("let f (a) = a end\nf(1, 2)");
    assert!(e.contains("Слишком много"), "сообщение: {}", e);
}

// #[test]
// fn function_too_few_args_error() {
//     let e = run_err("let f (a, b) = a + b end\nf(1)");
//     assert!(e.contains("Недостаточно"), "сообщение: {}", e);
// }
#[test]
fn function_too_few_args_curries() {
    // С каррированием недостаток аргументов возвращает функцию, а не ошибку
    assert_eq!(run("let f (a, b) = a + b end\necho. type f(1)"), "function\n");
    assert_eq!(run("let f (a, b) = a + b end\necho. f(1)(2)"), "3\n");
}

#[test]
fn lambda_works() {
    assert_eq!(run("let f = \\(x) = x * 2 end\necho. f(5)"), "10\n");
}

#[test]
fn if_expression_returns_value() {
    assert_eq!(run("echo. if 1 > 0 then \"да\" else \"нет\" end"), "да\n");
}

#[test]
fn if_elseif_else() {
    assert_eq!(
        run("let x = 0\necho. if x > 0 then \"+\" elseif x < 0 then \"-\" else \"0\" end"),
        "0\n"
    );
}

#[test]
fn ternary() {
    assert_eq!(run("echo. 1 if 2 > 1 else 2"), "1\n");
}

#[test]
fn ternary_chain_without_parens_error() {
    let e = run_err("echo. 1 if true else 2 if true else 3");
    assert!(e.contains("цепочка тернарных"), "сообщение: {}", e);
}

#[test]
fn for_num() {
    assert_eq!(
        run("for i = 1, 3 do echo. i end"),
        "1\n2\n3\n"
    );
}

#[test]
fn for_num_with_step() {
    assert_eq!(
        run("for i = 1, 5, 2 do echo. i end"),
        "1\n3\n5\n"
    );
}

#[test]
fn for_in_tuple() {
    assert_eq!(
        run("for v in (10, 20) do echo. v end"),
        "10\n20\n"
    );
}

#[test]
fn for_in_struct() {
    let src = "let s = { a = 1, b = 2 }\nfor k, v in s do echo. k, v end";
    assert_eq!(run(src), "a 1\nb 2\n");
}

#[test]
fn loop_break() {
    let src = "let ^i = 0\nloop\n^i = i + 1\nif i >= 3 then break end\nend\necho. i";
    assert_eq!(run(src), "3\n");
}

#[test]
fn loop_continue() {
    let src = "let ^sum = 0\nfor i = 1, 5 do\nif i == 3 then continue end\n^sum = sum + i\nend\necho. sum";
    assert_eq!(run(src), "12\n"); // 1+2+4+5
}

#[test]
fn tuple_index() {
    assert_eq!(run("let t = (10, 20, 30)\necho. t[0], t[1], t[2]"), "10 20 30\n");
}

#[test]
fn tuple_index_out_of_bounds() {
    let e = run_err("let t = (1, 2)\nt[5]");
    assert!(e.contains("вне кортежа"), "сообщение: {}", e);
}

#[test]
fn struct_field_access() {
    assert_eq!(run("let s = { name = \"Аня\", age = 12 }\necho. s.name, s.age"), "Аня 12\n");
}

#[test]
fn struct_missing_field_error() {
    let e = run_err("let s = { a = 1 }\ns.b");
    assert!(e.contains("отсутствует"), "сообщение: {}", e);
}

#[test]
fn destructuring() {
    assert_eq!(run("let (a, b) = (1, 2)\necho. a, b"), "1 2\n");
}

#[test]
fn match_literal() {
    let src = "let describe (x) =\nmatch x\n1 -> \"один\"\n2 -> \"два\"\n_ -> \"другое\"\nend\nend\necho. describe(1), describe(2), describe(9)";
    assert_eq!(run(src), "один два другое\n");
}

#[test]
fn match_guard() {
    let src = "let f (x) =\nmatch x\nn if n > 0 -> \"плюс\"\nn if n < 0 -> \"минус\"\n_ -> \"ноль\"\nend\nend\necho. f(5), f(-5), f(0)";
    assert_eq!(run(src), "плюс минус ноль\n");
}

#[test]
fn match_tuple_pattern() {
    let src = "let f (t) =\nmatch t\n(1, 2) -> \"один-два\"\n(a, b) -> \"другая пара\"\n_ -> \"не пара\"\nend\nend\necho. f((1, 2)), f((3, 4))";
    assert_eq!(run(src), "один-два другая пара\n");
}

#[test]
fn string_concat() {
    assert_eq!(run("echo. \"a\" ++ \"b\" ++ \"c\""), "abc\n");
}

#[test]
fn string_interpolation() {
    assert_eq!(run("let x = 2\necho. \"Результат: $(x + 3)\""), "Результат: 5\n");
}

#[test]
fn echo_no_newline() {
    assert_eq!(run("echo \"a\"\necho \"b\""), "ab");
}

#[test]
fn echo_dot_newline() {
    assert_eq!(run("echo. \"a\"\necho. \"b\""), "a\nb\n");
}

#[test]
fn numeric_literals() {
    assert_eq!(run("echo. 0b101010, 0o52, 0xFF, 0Xff"), "42 42 255 255\n");
    assert_eq!(run("echo. 0xFF_FF"), "65535\n");
}

#[test]
fn numeric_literal_bad() {
    let e = run_err("echo. 0b102");
    assert!(e.contains("Некорректное число"), "сообщение: {}", e);
}

// ============================================================
// builtin
// ============================================================

#[test]
fn builtin_len() {
    assert_eq!(run("echo. len(\"abc\"), len((1, 2)), len({ a = 1, b = 2 })"), "3 2 2\n");
}

#[test]
fn builtin_type() {
    assert_eq!(run("echo. type(1), type(\"a\"), type(nil)"), "number string nil\n");
}

#[test]
fn builtin_num() {
    assert_eq!(run("echo. num(\"42\") + 1"), "43\n");
}

#[test]
fn builtin_range() {
    assert_eq!(run("for i in range(0, 3) do echo. i end"), "0\n1\n2\n");
}

#[test]
fn num_parses_radix() {
    assert_eq!(run("echo. num(\"0xFF\")"), "255\n");
    assert_eq!(run("echo. num(\"0b101\")"), "5\n");
    assert_eq!(run("echo. num(\"0o10\")"), "8\n");
    assert_eq!(run("echo. num(\"42\")"), "42\n");
    assert_eq!(run("echo. num(\"3.14\")"), "3.14\n");
}

#[test]
fn bin_hex_oct() {
    assert_eq!(run("echo. bin(42), hex(255), oct(8)"), "0b101010 0xff 0o10\n");
    assert_eq!(run("echo. bin(-5)"), "-0b101\n");
    assert_eq!(run("echo. hex(0)"), "0x0\n");
}

#[test]
fn bin_float_error() {
    let e = run_err("bin(1.5)");
    assert!(e.contains("целое число"), "сообщение: {}", e);
}

#[test]
fn ord_chr() {
    assert_eq!(run("echo. ord(\"A\"), chr(65)"), "65 A\n");
    assert_eq!(run("echo. ord(\"€\"), chr(8364)"), "8364 €\n");
}

#[test]
fn ord_empty_error() {
    let e = run_err("ord(\"\")");
    assert!(e.contains("пуста"), "сообщение: {}", e);
}

#[test]
fn ord_too_long_error() {
    let e = run_err("ord(\"AB\")");
    assert!(e.contains("один символ"), "сообщение: {}", e);
}

#[test]
fn chr_invalid_error() {
    let e = run_err("chr(999999999)");
    assert!(e.contains("некорректный Unicode"), "сообщение: {}", e);
}

// ============================================================

#[test]
fn unclosed_function_error_mentions_start() {
    let e = run_err("let f (x) = x + 1");
    assert!(e.contains("начатой в строке"), "сообщение: {}", e);
}

#[test]
fn newline_between_exprs_required() {
    let e = run_err("echo. 1 echo. 2");
    assert!(e.contains("перевод строки"), "сообщение: {}", e);
}

#[test]
fn import_basic() {
    let dir = std::env::temp_dir().join(format!("simpl_import_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("utils.spl"), "let add (a, b) = a + b end\nlet PI = 3.14\n").unwrap();
    let main_src = "import utils\necho. utils.add(1, 2), utils.PI";
    let main_path = dir.join("main.spl");
    std::fs::write(&main_path, main_src).unwrap();

    let out = simpl::run_source_capture_with_path(main_src, Some(main_path), vec![]).unwrap();
    assert_eq!(out, "3 3.14\n");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_wildcard() {
    let dir = std::env::temp_dir().join(format!("simpl_import_w_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("m.spl"), "let f (x) = x * 2 end\n").unwrap();
    let main_src = "import m (*)\necho. f(5)";
    let main_path = dir.join("main.spl");
    std::fs::write(&main_path, main_src).unwrap();

    let out = simpl::run_source_capture_with_path(main_src, Some(main_path), vec![]).unwrap();
    assert_eq!(out, "10\n");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_alias() {
    let dir = std::env::temp_dir().join(format!("simpl_import_a_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("m.spl"), "let f (x) = x + 1 end\n").unwrap();
    let main_src = "import m as mm\necho. mm.f(5)";
    let main_path = dir.join("main.spl");
    std::fs::write(&main_path, main_src).unwrap();

    let out = simpl::run_source_capture_with_path(main_src, Some(main_path), vec![]).unwrap();
    assert_eq!(out, "6\n");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_missing_file_error() {
    let e = simpl::run_source_capture("import nosuchfile").expect_err("должна быть ошибка");
    assert!(e.contains("не удалось прочитать"), "сообщение: {}", e);
}

#[test]
fn import_cycle_error() {
    let dir = std::env::temp_dir().join(format!("simpl_import_cycle_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.spl"), "import b\nlet x = 1\n").unwrap();
    std::fs::write(dir.join("b.spl"), "import a\nlet y = 2\n").unwrap();
    let main_src = "import a";
    let main_path = dir.join("main.spl");
    std::fs::write(&main_path, main_src).unwrap();

    let e = simpl::run_source_capture_with_path(main_src, Some(main_path), vec![]).expect_err("должна быть ошибка");
    assert!(e.contains("Циклический импорт"), "сообщение: {}", e);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_inside_function_error() {
    let e = simpl::run_source_capture("let f () =\nimport x\n1\nend").expect_err("должна быть ошибка");
    assert!(e.contains("только на верхнем уровне"), "сообщение: {}", e);
}

#[test]
fn single_quoted_string() {
    assert_eq!(run("echo. 'привет'"), "привет\n");
}

#[test]
fn single_quoted_with_double_inside() {
    assert_eq!(run("echo. 'он сказал \"привет\"'"), "он сказал \"привет\"\n");
}

#[test]
fn double_quoted_with_single_inside() {
    assert_eq!(run("echo. \"it's fine\""), "it's fine\n");
}

#[test]
fn single_quoted_escape() {
    assert_eq!(run("echo. 'it\\'s fine'"), "it's fine\n");
}

#[test]
fn single_quoted_interpolation() {
    assert_eq!(run("let x = 2\necho. 'сумма: $(x + 3)'"), "сумма: 5\n");
}

#[test]
fn type_operator() {
    assert_eq!(run("echo. type 42"), "number\n");
    assert_eq!(run("echo. type \"abc\""), "string\n");
    assert_eq!(run("echo. type true"), "bool\n");
    assert_eq!(run("echo. type nil"), "nil\n");
    assert_eq!(run("echo. type (1, 2)"), "tuple\n");
    assert_eq!(run("echo. type { a = 1 }"), "struct\n");
}

#[test]
fn type_with_parens_backward_compat() {
    assert_eq!(run("echo. type(42)"), "number\n");
}

#[test]
fn type_precedence() {
    // type x + 1 = (type x) + 1 → "number" + 1 → ошибка, потому что "number" ++ 1 нельзя через +
    // Проверим через конкатенацию:
    assert_eq!(run("let x = 42\necho. type x ++ \"!\""), "number!\n");
}

#[test]
fn type_of_function() {
    assert_eq!(run("let f (x) = x end\necho. type f"), "function\n");
}

#[test]
fn type_in_expression() {
    assert_eq!(run("echo. type 42 == \"number\""), "true\n");
}

#[test]
fn type_is_keyword_not_value() {
    let e = run_err("let t = type");
    assert!(e.contains("ожидалось выражение"), "сообщение: {}", e);
}

#[test]
fn redeclare_in_same_scope_error() {
    let e = run_err("let x = 1\nlet x = 2");
    assert!(e.contains("уже объявлена"), "сообщение: {}", e);
}

#[test]
fn redeclare_in_nested_scope_ok() {
    let src = "let x = 1\nif true then\nlet x = 2\necho. x\nend\necho. x";
    assert_eq!(run(src), "2\n1\n");
}

#[test]
fn redeclare_in_function_ok() {
    let src = "let x = 1\nlet f () =\nlet x = 2\nx\nend\necho. f(), x";
    assert_eq!(run(src), "2 1\n");
}

#[test]
fn redeclare_after_assign_ok() {
    // После let ^x = 1; ^x = 2 — x изменена, но не переобъявлена
    let src = "let ^x = 1\n^x = 2\necho. x";
    assert_eq!(run(src), "2\n");
}

#[test]
fn duplicate_param_error() {
    let e = run_err("let f (x, x) = x end");
    assert!(e.contains("уже объявлен"), "сообщение: {}", e);
}

#[test]
fn let_tuple_redeclare_error() {
    let e = run_err("let (a, b) = (1, 2)\nlet (a, c) = (3, 4)");
    assert!(e.contains("уже объявлена"), "сообщение: {}", e);
}

#[test]
fn use_conflict_error() {
    let src = "let trim = 1\nuse string (trim)";
    let e = run_err(src);
    assert!(e.contains("уже объявлена"), "сообщение: {}", e);
}

// ============================================================
// argv
// ============================================================

#[test]
fn argv_empty() {
    let out = simpl::run_source_capture("echo. argv").unwrap();
    assert_eq!(out, "()\n");
}

#[test]
fn argv_with_args() {
    let out = simpl::run_source_capture_with_argv(
        "echo. argv",
        vec!["hello".into(), "world".into()],
    ).unwrap();
    assert_eq!(out, "(hello, world)\n");
}

#[test]
fn argv_len_and_index() {
    let out = simpl::run_source_capture_with_argv(
        "echo. len(argv), argv[0], argv[1]",
        vec!["a".into(), "b".into()],
    ).unwrap();
    assert_eq!(out, "2 a b\n");
}

#[test]
fn argv_can_be_shadowed_in_function() {
    let out = simpl::run_source_capture_with_argv(
        "let f (*argv) = len(argv) end\necho. f(1, 2, 3), len(argv)",
        vec!["a".into()],
    ).unwrap();
    assert_eq!(out, "3 1\n");
}

#[test]
fn sleep_zero_no_delay() {
    let start = std::time::Instant::now();
    let out = run("sleep(0)\necho. \"ok\"");
    assert_eq!(out, "ok\n");
    assert!(start.elapsed().as_millis() < 100, "sleep(0) не должен спать");
}

#[test]
fn sleep_short() {
    let start = std::time::Instant::now();
    let out = run("sleep(0.1)\necho. \"ok\"");
    assert_eq!(out, "ok\n");
    let ms = start.elapsed().as_millis();
    assert!(ms >= 90 && ms < 2000, "sleep(0.1) спал {}мс", ms);
}

#[test]
fn sleep_negative_error() {
    let e = run_err("sleep(-1)");
    assert!(e.contains("неотрицательное"), "сообщение: {}", e);
}

#[test]
fn sleep_too_long_error() {
    let e = run_err("sleep(1000000000000000000000000000000000000000000000)");
    assert!(e.contains("разумное"), "сообщение: {}", e);
}

#[test]
fn sleep_bad_type_error() {
    let e = run_err("sleep(\"1\")");
    assert!(e.contains("число"), "сообщение: {}", e);
}

#[test]
fn sleep_wrong_arity_error() {
    let e = run_err("sleep()");
    assert!(e.contains("ожидает 1"), "сообщение: {}", e);
}

#[test]
fn curry_partial_apply() {
    assert_eq!(run("let add (a, b) = a + b end\necho. add(1)(2)"), "3\n");
}

#[test]
fn curry_full_call() {
    assert_eq!(run("let add (a, b) = a + b end\necho. add(1, 2)"), "3\n");
}

#[test]
fn curry_partial_is_function() {
    assert_eq!(run("let add (a, b) = a + b end\necho. type add(1)"), "function\n");
}

#[test]
fn curry_three_args() {
    assert_eq!(run("let f (a, b, c) = a + b + c end\necho. f(1)(2)(3), f(1, 2, 3), f(1, 2)(3), f(1)(2, 3)"), "6 6 6 6\n");
}

#[test]
fn curry_default_not_curried() {
    assert_eq!(run("let g (a, b = 10) = a + b end\necho. g(1)"), "11\n");
}

#[test]
fn curry_default_curried_when_required_missing() {
    assert_eq!(run("let g (a, b = 10) = a + b end\necho. g()(5)"), "15\n");
}

#[test]
fn curry_star_args() {
    assert_eq!(run("let h (a, *rest) = (a, rest) end\necho. h(1), h(1, 2, 3)"), "(1, ()) (1, (2, 3))\n");
}

#[test]
fn curry_star_args_curried() {
    assert_eq!(run("let h (a, *rest) = (a, rest) end\necho. h()(5)"), "(5, ())\n");
}

#[test]
fn curry_too_many_args_error() {
    let e = run_err("let add (a, b) = a + b end\nadd(1, 2, 3)");
    assert!(e.contains("Слишком много"), "сообщение: {}", e);
}

#[test]
fn curry_chain() {
    assert_eq!(run("let add (a, b) = a + b end\nlet add5 = add(5)\necho. add5(3), add5(10)"), "8 15\n");
}

#[test]
fn curry_lambda() {
    assert_eq!(run("let f = \\(a, b) = a * b end\necho. f(3)(4)"), "12\n");
}

#[test]
fn pipe_basic() {
    assert_eq!(run("let inc (n) = n + 1 end\necho. 1 $ inc"), "2\n");
}

#[test]
fn pipe_chain() {
    let src = "let inc (n) = n + 1 end\nlet dbl (n) = n * 2 end\nlet add (a, b) = a + b end\nlet x = 1\necho. x $ inc $ dbl $ add(10) $ inc";
    assert_eq!(run(src), "15\n");
}

#[test]
fn compose_basic() {
    assert_eq!(run("let inc (n) = n + 1 end\nlet dbl (n) = n * 2 end\nlet f = inc .. dbl\necho. f(3)"), "8\n");
}

#[test]
fn compose_chain() {
    let src = "let inc (n) = n + 1 end\nlet dbl (n) = n * 2 end\nlet add (a, b) = a + b end\nlet f = inc .. dbl .. add(10) .. inc\necho. f(1)";
    assert_eq!(run(src), "15\n");
}

#[test]
fn compose_non_function_error() {
    let e = run_err("let f = 1 .. 2");
    assert!(e.contains("Композиция"), "сообщение: {}", e);
}

#[test]
fn pipe_non_function_error() {
    let e = run_err("echo. 1 $ 2");
    assert!(e.contains("Нельзя вызвать"), "сообщение: {}", e);
}

#[test]
fn flip_basic() {
    assert_eq!(run("let sub (a, b) = a - b end\nlet rsub = flip(sub)\necho. rsub(3, 10)"), "7\n");
}

#[test]
fn flip_curried() {
    assert_eq!(run("let sub (a, b) = a - b end\necho. flip(sub)(1)(2)"), "1\n");
}

#[test]
fn flip_curried_full() {
    assert_eq!(run("let sub (a, b) = a - b end\necho. flip(sub)(1, 2)"), "1\n");
}

#[test]
fn flip_non_function_error() {
    let e = run_err("flip(42)");
    assert!(e.contains("ожидалась функция"), "сообщение: {}", e);
}

#[test]
fn flip_unary_error() {
    let e = run_err("let inc (x) = x + 1 end\nflip(inc)");
    assert!(e.contains("двумя обязательными"), "сообщение: {}", e);
}

#[test]
fn flip_builtin_error() {
    let e = run_err("flip(len)");
    assert!(e.contains("встроенные"), "сообщение: {}", e);
}

#[test]
fn pipe_with_flip() {
    assert_eq!(run("let sub (a, b) = a - b end\necho. 10 $ flip(sub)(3)"), "7\n");
}

#[test]
fn comp_basic() {
    let src = "let squares = { x: x*x for x in range(1, 10) }\necho. squares[3], squares[5]";
    assert_eq!(run(src), "9 25\n");
}

#[test]
fn comp_with_if() {
    let src = "let evens = { x: x for x in range(0, 10) if x % 2 == 0 }\necho. struct.values(evens)";
    assert_eq!(run(src), "(0, 2, 4, 6, 8)\n");
}

#[test]
fn comp_kv_struct() {
    let src = "let p = { a = 1, b = 2 }\nlet m = { k: v*10 for k, v in p }\necho. m.a, m.b";
    assert_eq!(run(src), "10 20\n");
}

#[test]
fn comp_kv_tuple() {
    let src = "let t = (10, 20, 30)\nlet m = { i: v for i, v in t }\necho. m[0], m[1], m[2]";
    assert_eq!(run(src), "10 20 30\n");
}

#[test]
fn comp_empty_result() {
    let src = "let m = { x: x for x in range(0, 3) if x > 100 }\necho. len(m)";
    assert_eq!(run(src), "0\n");
}

#[test]
fn comp_overwrite() {
    // одинаковые ключи для разных x — перезапись
    let src = "let m = { 1: x for x in range(0, 5) }\necho. m[1]";
    assert_eq!(run(src), "4\n");
}

#[test]
fn comp_without_for_error() {
    let e = run_err("let m = { a: 1 }");
    assert!(e.contains("ожидался 'for'"), "сообщение: {}", e);
}

#[test]
fn comp_bad_iter_error() {
    let e = run_err("let m = { x: x for x in 42 }");
    assert!(e.contains("кортеж или struct"), "сообщение: {}", e);
}

#[test]
fn comp_duplicate_var_error() {
    let e = run_err("let m = { x: x for x, x in (1, 2) }");
    assert!(e.contains("уже объявлена"), "сообщение: {}", e);
}

#[test]
fn literal_with_eq_still_works() {
    let src = "let s = { a = 1, b = 2 }\necho. s.a, s.b";
    assert_eq!(run(src), "1 2\n");
}

#[test]
fn literal_auto_keys_still_works() {
    let src = "let s = { 10, 20, 30 }\necho. s[0], s[2]";
    assert_eq!(run(src), "10 30\n");
}

#[test]
fn bit_and() {
    assert_eq!(run("echo. 6 & 3"), "2\n");
}

#[test]
fn bit_or() {
    assert_eq!(run("echo. 6 | 3"), "7\n");
}

#[test]
fn bit_xor() {
    assert_eq!(run("echo. 6 xor 3"), "5\n");
}

#[test]
fn bit_not() {
    assert_eq!(run("echo. ~5"), "-6\n");
}

#[test]
fn shift_left() {
    assert_eq!(run("echo. 1 << 4"), "16\n");
}

#[test]
fn shift_right() {
    assert_eq!(run("echo. 16 >> 2"), "4\n");
}

#[test]
fn shift_right_negative() {
    assert_eq!(run("echo. -8 >> 1"), "-4\n");
}

#[test]
fn bitop_float_error() {
    let e = run_err("echo. 3.5 & 1");
    assert!(e.contains("целое число"), "сообщение: {}", e);
}

#[test]
fn bitop_overflow_error() {
    let e = run_err("echo. 9007199254740992 & 1");  // 2^53
    assert!(e.contains("вне диапазона"), "сообщение: {}", e);
}

#[test]
fn shift_too_big_error() {
    let e = run_err("echo. 1 << 64");
    assert!(e.contains("сдвиг на 64"), "сообщение: {}", e);
}

#[test]
fn shift_negative_error() {
    let e = run_err("echo. 1 << -1");
    assert!(e.contains("сдвиг на -1"), "сообщение: {}", e);
}

#[test]
fn bitop_precedence() {
    // 6 & 3 | 2 = (6 & 3) | 2 = 2 | 2 = 2
    assert_eq!(run("echo. 6 & 3 | 2"), "2\n");
    // 1 << 2 + 1 = 1 << (2 + 1) = 8 (сдвиг ниже +)
    assert_eq!(run("echo. 1 << 2 + 1"), "8\n");
    // 6 xor 3 & 1 = 6 xor (3 & 1) = 6 xor 1 = 7
    assert_eq!(run("echo. 6 xor 3 & 1"), "7\n");
}

#[test]
fn logical_and_or_not_still_work() {
    assert_eq!(run("echo. true and false, true or false, not false"), "false true true\n");
}

#[test]
fn logical_vs_bitwise() {
    // and — логическое (truthy), & — побитовое (только числа)
    assert_eq!(run("echo. 1 and 2"), "true\n");
    assert_eq!(run("echo. 1 & 2"), "0\n");
}

//================================================================
#[test]
fn number_exp_basic() {
    assert_eq!(run("echo. 1e2"), "100\n");
    assert_eq!(run("echo. 1e20"), "100000000000000000000\n");
}

#[test]
fn number_exp_fraction() {
    assert_eq!(run("echo. 1.5e-3"), "0.0015\n");
    assert_eq!(run("echo. 2e+3"), "2000\n");
}

#[test]
fn number_exp_underscores() {
    assert_eq!(run("echo. 1_000e3"), "1000000\n");
}

#[test]
fn number_exp_not_consumed() {
    // 1e — не экспонента, 1 и Ident("e") → ошибка "переменная e не объявлена"
    let e = run_err("echo. 1e");
    assert!(e.contains("не объявлена") || e.contains("перевод строки"), "сообщение: {}", e);
}

#[test]
fn number_exp_after_hex_not_applied() {
    // 0x1e20 — hex, не экспонента
    assert_eq!(run("echo. 0x1e20"), "7712\n");
}

#[test]
fn bitop_overflow_error_for_exponent() {
    // теперь 1e20 парсится, as_i64 отвергает
    let e = run_err("echo. 1e20 & 1");
    assert!(e.contains("вне диапазона"), "сообщение: {}", e);
}

// ============================================================
// checker
// ============================================================

#[test]
fn check_undefined_var() {
    let e = run_err("echo. x");
    assert!(e.contains("не объявлена"), "сообщение: {}", e);
}

#[test]
fn check_assign_undefined() {
    let e = run_err("^x = 1");
    assert!(e.contains("не объявлена"), "сообщение: {}", e);
}

#[test]
fn check_assign_immutable() {
    let e = run_err("let x = 1\n^x = 2");
    assert!(e.contains("неизменяемая"), "сообщение: {}", e);
}

#[test]
fn check_var_already_defined() {
    let e = run_err("let x = 1\nlet x = 2");
    assert!(e.contains("уже объявлена"), "сообщение: {}", e);
}

#[test]
fn check_break_outside_loop() {
    let e = run_err("break");
    assert!(e.contains("break"), "сообщение: {}", e);
}

#[test]
fn check_continue_outside_loop() {
    let e = run_err("continue");
    assert!(e.contains("continue"), "сообщение: {}", e);
}

#[test]
fn check_use_unknown_module() {
    let e = run_err("use foo (*)");
    assert!(e.contains("неизвестная библиотека"), "сообщение: {}", e);
}

#[test]
fn check_use_name_not_in_module() {
    let e = run_err("use string (nosuch)");
    assert!(e.contains("нет функции"), "сообщение: {}", e);
}

#[test]
fn check_flip_builtin() {
    let e = run_err("flip(len)");
    assert!(e.contains("встроенные"), "сообщение: {}", e);
}

#[test]
fn check_compose_non_function() {
    let e = run_err("let f = 1 .. 2");
    assert!(e.contains("Композиция"), "сообщение: {}", e);
}

#[test]
fn check_too_many_args_known_fn() {
    let e = run_err("let f (a, b) = a + b end\nf(1, 2, 3)");
    assert!(e.contains("Слишком много"), "сообщение: {}", e);
}

#[test]
fn check_bi_arity_too_few() {
    let e = run_err("len()");
    assert!(e.contains("len()"), "сообщение: {}", e);
}

#[test]
fn check_bi_arity_too_many() {
    let e = run_err("len(1, 2)");
    assert!(e.contains("len()"), "сообщение: {}", e);
}

#[test]
fn check_bi_unknown() {
    let e = run_err("nosuch()");
    assert!(e.contains("Неизвестная функция"), "сообщение: {}", e);
}

#[test]
fn check_call_non_function() {
    let e = run_err("42(1)");
    assert!(e.contains("Нельзя вызвать"), "сообщение: {}", e);
}

#[test]
fn check_for_step_zero() {
    let e = run_err("for i = 1, 10, 0 do end");
    assert!(e.contains("Шаг for"), "сообщение: {}", e);
}

#[test]
fn check_index_out_of_bounds() {
    let e = run_err("echo. (1, 2)[5]");
    assert!(e.contains("вне кортежа"), "сообщение: {}", e);
}

#[test]
fn check_import_cycle() {
    let dir = std::env::temp_dir().join(format!("simpl_check_cycle_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.spl"), "import b\nlet x = 1\n").unwrap();
    std::fs::write(dir.join("b.spl"), "import a\nlet y = 2\n").unwrap();
    let main_src = "import a";
    let main_path = dir.join("main.spl");
    std::fs::write(&main_path, main_src).unwrap();

    let e = simpl::run_source_capture_with_path(main_src, Some(main_path), vec![]).expect_err("должна быть ошибка");
    assert!(e.contains("Циклический импорт"), "сообщение: {}", e);

    let _ = std::fs::remove_dir_all(&dir);
}