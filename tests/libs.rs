use simpl::run_source_capture;

fn run(src: &str) -> String {
    run_source_capture(src).expect("программа должна выполниться без ошибок")
}

fn run_err(src: &str) -> String {
    run_source_capture(src).expect_err("программа должна упасть с ошибкой")
}

// ============================================================
// use
// ============================================================

#[test]
fn use_wildcard() {
    let src = "use string (*)\necho. upper(\"abc\")";
    assert_eq!(run(src), "ABC\n");
}

#[test]
fn use_explicit() {
    let src = "use string (trim)\necho. trim(\"  hi  \"), string.upper(\"x\")";
    assert_eq!(run(src), "hi X\n");
}

#[test]
fn use_alias_only() {
    let src = "use string as s\necho. s.upper(\"abc\")";
    assert_eq!(run(src), "ABC\n");
}

#[test]
fn use_alias_with_list() {
    let src = "use string as s (trim)\necho. trim(\"  hi  \"), s.upper(\"x\")";
    assert_eq!(run(src), "hi X\n");
}

#[test]
fn use_conflict_error() {
    let src = "let trim = 1\nuse string (trim)";
    let e = run_err(src);
    assert!(e.contains("уже занято"), "сообщение: {}", e);
}

#[test]
fn use_unknown_module_error() {
    let e = run_err("use foo (*)");
    assert!(e.contains("неизвестная библиотека"), "сообщение: {}", e);
}

#[test]
fn use_unknown_name_error() {
    let e = run_err("use string (nosuch)");
    assert!(e.contains("нет функции"), "сообщение: {}", e);
}

#[test]
fn use_inside_function_error() {
    let src = "let f () =\nuse string (*)\n1\nend";
    let e = run_err(src);
    assert!(e.contains("только на верхнем уровне"), "сообщение: {}", e);
}

// ============================================================
// string
// ============================================================

#[test]
fn string_upper_lower() {
    assert_eq!(run("echo. string.upper(\"abc\"), string.lower(\"XYZ\")"), "ABC xyz\n");
}

#[test]
fn string_trim() {
    assert_eq!(run("echo. \"[\" ++ string.trim(\"  hi  \") ++ \"]\""), "[hi]\n");
}

#[test]
fn string_rep() {
    assert_eq!(run("echo. string.rep(\"ab\", 3)"), "ababab\n");
}

#[test]
fn string_sub() {
    assert_eq!(run("echo. string.sub(\"hello\", 1, 3)"), "el\n");       // [1,3) → "el"
    assert_eq!(run("echo. string.sub(\"hello\", 2)"), "llo\n");         // [2,len) → "llo"
    assert_eq!(run("echo. string.sub(\"hello\", 1, 4)"), "ell\n");      // [1,4) → "ell"
}

#[test]
fn string_find() {
    assert_eq!(run("echo. string.find(\"hello\", \"ll\")"), "2\n");
    assert_eq!(run("echo. string.find(\"hello\", \"zz\")"), "-1\n");
}

#[test]
fn string_split_join() {
    assert_eq!(
        run("let parts = string.split(\"a,b,c\", \",\")\necho. string.join(parts, \"-\")"),
        "a-b-c\n"
    );
}

#[test]
fn string_starts_ends_with() {
    assert_eq!(
        run("echo. string.starts_with(\"hello\", \"he\"), string.ends_with(\"hello\", \"lo\")"),
        "true true\n"
    );
}

#[test]
fn string_chars_from_tuple() {
    assert_eq!(run("echo. string.from_tuple(string.chars(\"abc\"))"), "abc\n");
}

#[test]
fn string_split_empty_sep_error() {
    let e = run_err("string.split(\"abc\", \"\")");
    assert!(e.contains("пустой строкой"), "сообщение: {}", e);
}

// ============================================================
// math
// ============================================================

#[test]
fn math_abs_floor_ceil_round() {
    assert_eq!(
        run("echo. math.abs(-5), math.floor(2.7), math.ceil(2.1), math.round(2.5)"),
        "5 2 3 3\n"
    );
}

#[test]
fn math_sqrt() {
    assert_eq!(run("echo. math.sqrt(16)"), "4\n");
}

#[test]
fn math_sqrt_negative_error() {
    let e = run_err("math.sqrt(-1)");
    assert!(e.contains("отрицательным"), "сообщение: {}", e);
}

#[test]
fn math_min_max() {
    assert_eq!(
        run("echo. math.min(3, 1, 2), math.max(3, 1, 2)"),
        "1 3\n"
    );
}

#[test]
fn math_min_empty_error() {
    let e = run_err("math.min()");
    assert!(e.contains("хотя бы один"), "сообщение: {}", e);
}

#[test]
fn math_random_range() {
    let src = "let ^ok = true\nfor i = 1, 100 do\nlet r = math.random(10)\nif r < 0 then ^ok = false end\nif r >= 10 then ^ok = false end\nend\necho. ok";
    assert_eq!(run(src), "true\n");
}

// ============================================================
// iter (map/filter/reduce)
// ============================================================

#[test]
fn iter_map_tuple() {
    assert_eq!(
        run("echo. iter.map((1, 2, 3), \\(x) = x * 2 end)"),
        "(2, 4, 6)\n"
    );
}

#[test]
fn iter_map_struct_array() {
    let src = "let s = { 10, 20, 30 }\nlet m = iter.map(s, \\(x) = x + 1 end)\necho. struct.values(m)";
    assert_eq!(run(src), "(11, 21, 31)\n");
}

#[test]
fn iter_map_non_array_struct_error() {
    let e = run_err("iter.map({ a = 1 }, \\(x) = x end)");
    assert!(e.contains("struct-массивами"), "сообщение: {}", e);
}

#[test]
fn iter_filter_tuple() {
    assert_eq!(
        run("echo. iter.filter((1, 2, 3, 4), \\(x) = x % 2 == 0 end)"),
        "(2, 4)\n"
    );
}

#[test]
fn iter_filter_struct_array_reindexes() {
    let src = "let s = { 1, 2, 3, 4 }\nlet out = iter.filter(s, \\(x) = x % 2 == 0 end)\necho. struct.values(out)";
    assert_eq!(run(src), "(2, 4)\n");
}

#[test]
fn iter_reduce_tuple() {
    assert_eq!(
        run("echo. iter.reduce((1, 2, 3, 4), \\(acc, x) = acc + x end, 0)"),
        "10\n"
    );
}

#[test]
fn iter_reduce_struct_array() {
    let src = "let s = { 10, 20, 30 }\necho. iter.reduce(s, \\(acc, x) = acc + x end, 0)";
    assert_eq!(run(src), "60\n");
}

#[test]
fn iter_map_non_function_error() {
    let e = run_err("iter.map((1, 2), 42)");
    assert!(e.contains("ожидалась функция"), "сообщение: {}", e);
}

// ============================================================
// tuple
// ============================================================

#[test]
fn tuple_get_ok() {
    assert_eq!(run("echo. tuple.get((10, 20), 1)"), "20\n");
}

#[test]
fn tuple_get_default() {
    assert_eq!(run("echo. tuple.get((10, 20), 5, 0)"), "0\n");
}

#[test]
fn tuple_get_oob_error() {
    let e = run_err("tuple.get((10, 20), 5)");
    assert!(e.contains("вне длины"), "сообщение: {}", e);
}

#[test]
fn tuple_slice() {
    assert_eq!(
        run("echo. tuple.slice((10, 20, 30, 40), 1, 3)"),
        "(20, 30)\n"
    );
}

#[test]
fn tuple_concat_push() {
    assert_eq!(
        run("echo. tuple.concat((1, 2), (3, 4)), tuple.push((1, 2), 3)"),
        "(1, 2, 3, 4) (1, 2, 3)\n"
    );
}

#[test]
fn tuple_reverse() {
    assert_eq!(run("echo. tuple.reverse((1, 2, 3))"), "(3, 2, 1)\n");
}

#[test]
fn tuple_index_of_contains() {
    assert_eq!(
        run("echo. tuple.index_of((10, 20, 30), 20), tuple.contains((1, 2), 2)"),
        "1 true\n"
    );
}

#[test]
fn tuple_sort() {
    assert_eq!(run("echo. tuple.sort((3, 1, 2))"), "(1, 2, 3)\n");
}

#[test]
fn tuple_zip() {
    assert_eq!(
        run("echo. tuple.zip((1, 2), (10, 20))"),
        "((1, 10), (2, 20))\n"
    );
}

#[test]
fn tuple_zip_mismatch_error() {
    let e = run_err("tuple.zip((1, 2), (1,))");
    assert!(e.contains("не совпадают"), "сообщение: {}", e);
}

// ============================================================
// struct
// ============================================================

#[test]
fn struct_keys_values() {
    assert_eq!(
        run("let s = { a = 1, b = 2 }\necho. struct.keys(s), struct.values(s)"),
        "(a, b) (1, 2)\n"
    );
}

#[test]
fn struct_entries() {
    assert_eq!(
        run("let s = { x = 1 }\necho. struct.entries(s)"),
        "((x, 1))\n"
    );
}

#[test]
fn struct_has() {
    assert_eq!(
        run("let s = { a = 1 }\necho. struct.has(s, \"a\"), struct.has(s, \"b\")"),
        "true false\n"
    );
}

#[test]
fn struct_merge() {
    let src = "let a = { x = 1 }\nlet b = { y = 2 }\nlet m = struct.merge(a, b)\necho. m.x, m.y";
    assert_eq!(run(src), "1 2\n");
}

#[test]
fn struct_merge_overwrites() {
    assert_eq!(
        run("let a = { x = 1 }\nlet b = { x = 2 }\necho. struct.merge(a, b).x"),
        "2\n"
    );
}

#[test]
fn struct_remove() {
    let src = "let s = { a = 1, b = 2 }\nlet r = struct.remove(s, \"a\")\necho. struct.has(r, \"a\"), r.b";
    assert_eq!(run(src), "false 2\n");
}

#[test]
fn struct_from_tuple() {
    assert_eq!(
        run("echo. struct.from_tuple((\"a\", \"b\"))"),
        "{ 0 = a, 1 = b }\n"
    );
}

#[test]
fn struct_from_pairs() {
    let src = "let s = struct.from_pairs(((\"a\", 1), (\"b\", 2)))\necho. s.a, s.b";
    assert_eq!(run(src), "1 2\n");
}

#[test]
fn struct_zip() {
    let src = "let s = struct.zip((\"a\", \"b\"), (1, 2))\necho. s.a, s.b";
    assert_eq!(run(src), "1 2\n");
}
// ============================================================
// file
// ============================================================

fn tmp_path(name: &str) -> String {
    let mut p = std::env::temp_dir();
    p.push(format!("simpl_test_{}_{}.txt", name, std::process::id()));
    p.to_string_lossy().replace('\\', "/")
}

#[test]
fn file_write_read_exists() {
    let path = tmp_path("rw");
    let src = format!(
        "file.write(\"{p}\", \"hello\")\necho. file.exists(\"{p}\")\necho. file.read(\"{p}\")",
        p = path
    );
    assert_eq!(run(&src), "true\nhello\n");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn file_append() {
    let path = tmp_path("append");
    let src = format!(
        "file.write(\"{p}\", \"a\")\nfile.append(\"{p}\", \"b\")\necho. file.read(\"{p}\")",
        p = path
    );
    assert_eq!(run(&src), "ab\n");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn file_exists_false() {
    let path = tmp_path("no_such_file_xyz");
    let src = format!("echo. file.exists(\"{}\")", path);
    assert_eq!(run(&src), "false\n");
}

#[test]
fn file_read_missing_error() {
    let path = tmp_path("missing");
    let src = format!("file.read(\"{}\")", path);
    let e = run_err(&src);
    assert!(e.contains("не удалось прочитать"), "сообщение: {}", e);
}

// ============================================================
// datetime
// ============================================================

#[test]
fn datetime_now_fields() {
    let src = "let d = datetime.now()\necho. type(d), type(d.year), type(d.month), type(d.day)";
    assert_eq!(run(src), "struct number number number\n");
}

#[test]
fn datetime_format() {
    let src = "let d = { year = 2026, month = 10, day = 21, hour = 14, min = 30, sec = 0 }\necho. datetime.format(d, \"%Y-%m-%d %H:%M:%S\")";
    assert_eq!(run(src), "2026-10-21 14:30:00\n");
}

#[test]
fn datetime_timestamp_roundtrip() {
    let src = "let d = { year = 2026, month = 1, day = 1, hour = 0, min = 0, sec = 0 }\nlet ts = datetime.timestamp(d)\nlet d2 = datetime.from_timestamp(ts)\necho. datetime.format(d2, \"%Y-%m-%d %H:%M:%S\")";
    // локальное время может отличаться от UTC, но roundtrip через timestamp должен вернуть то же локальное время
    assert_eq!(run(src), "2026-01-01 00:00:00\n");
}

#[test]
fn datetime_bad_struct_error() {
    let e = run_err("datetime.format({ year = 2026 }, \"%Y\")");
    assert!(e.contains("должно быть числом"), "сообщение: {}", e);
}