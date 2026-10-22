# Changelog

## 1.5.0

- Добавлена подсветка `import`: ключевое слово (`keyword.control.import.simpl`), путь модуля (`entity.name.namespace.simpl`), путь в кавычках (`string.quoted.double.path.simpl`).
- `import` добавлен в список ключевых слов на случай, если специфичный паттерн не сработал.
- Обновлён `README.md`: раздел про `import` и новые скоупы.

## 1.4.0

- Улучшена подсветка `echo` и `echo.` — теперь отдельные скоупы `keyword.other.echo.simpl` и `keyword.other.echo.newline.simpl`.
- Добавлена подсветка встроенных модулей: `string`, `math`, `tuple`, `struct`, `iter`, `file`, `datetime` (`support.class.module.simpl`).
- Добавлена подсветка встроенных глобальных функций: `len`, `type`, `str`, `num`, `range`, `input` (`support.function.builtin.simpl`).
- Добавлена подсветка объявлений пользовательских функций: имя после `let` перед `(` (`entity.name.function.simpl`).
- Добавлена подсветка лямбд `\(...)`.
- Добавлена подсветка ключевых слов `use`, `as`.
- Добавлена подсветка `^` как `keyword.operator.mutability.simpl`.
- Добавлена подсветка `\` как `keyword.operator.lambda.simpl`.
- Добавлены автоотступы для `if`, `for`, `loop`, `match`, `let ... (...)`.
- Добавлены `README.md` и `CHANGELOG.md`.

## 1.0.0

- Первая версия: базовая подсветка синтаксиса Simpl.