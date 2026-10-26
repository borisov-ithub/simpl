// ============================================================
// check_source
// ============================================================

#[test]
fn check_source_ok() {
    assert!(simpl::check_source("let x = 1").is_ok());
}

#[test]
fn check_source_parse_error() {
    let e = simpl::check_source("let x = ").expect_err("должна быть ошибка");
    assert!(e.contains("ожидалось выражение"), "сообщение: {}", e);
}

#[test]
fn check_source_does_not_execute() {
    // Проверяем, что check_source не выполняет код: деление на ноль не должно упасть,
    // потому что оно только в рантайме, а парсер его не трогает.
    assert!(simpl::check_source("let x = 1 / 0").is_ok());
}

#[test]
fn cli_passes_args_to_program() {
    use std::process::Command;
    let dir = std::env::temp_dir().join(format!("simpl_argv_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("t.spl");
    std::fs::write(&script, "echo. argv").unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_simpl"))
        .arg(&script)
        .arg("hello")
        .arg("world")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "(hello, world)\n");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_flag_after_file_is_program_arg() {
    use std::process::Command;
    let dir = std::env::temp_dir().join(format!("simpl_argv2_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("t.spl");
    std::fs::write(&script, "echo. argv").unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_simpl"))
        .arg(&script)
        .arg("-v")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "(-v)\n");

    let _ = std::fs::remove_dir_all(&dir);
}