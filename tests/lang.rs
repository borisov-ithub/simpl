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

#[test]
fn function_too_few_args_error() {
    let e = run_err("let f (a, b) = a + b end\nf(1)");
    assert!(e.contains("Недостаточно"), "сообщение: {}", e);
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
fn unclosed_function_error_mentions_start() {
    let e = run_err("let f (x) = x + 1");
    assert!(e.contains("начатой в строке"), "сообщение: {}", e);
}

#[test]
fn newline_between_exprs_required() {
    let e = run_err("echo. 1 echo. 2");
    assert!(e.contains("перевод строки"), "сообщение: {}", e);
}