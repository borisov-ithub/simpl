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