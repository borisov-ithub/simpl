# Simpl для VSCode

Расширение для языка программирования **Simpl** — учебного языка с Lua-подобным синтаксисом.

Версия: **2.2.4**

## Что внутри

- Подсветка синтаксиса для файлов `.spl`.
- Различение `echo` (без перевода строки) и `echo.` (с переводом строки).
- Подсветка встроенных модулей: `string`, `math`, `tuple`, `struct`, `iter`, `file`, `datetime`.
- Подсветка встроенных глобальных функций: `len`, `type`, `str`, `num`, `range`, `input`.
- Подсветка объявлений пользовательских функций: `let name (...) = ... end`.
- Подсветка лямбд: `\(...) = ... end`.
- Подсветка строк в двойных и одинарных кавычках.
- Подсветка интерполяции в строках: `"Сумма: $(2 + 2)"`.
- Автоматическое закрытие скобок и кавычек.
- Автоотступы для `if`, `for`, `loop`, `match`, `let ... (...)`.
- Подсветка `import`: ключевое слово, путь модуля и путь в кавычках.

## Установка

### Через `.vsix`

1. Скачайте `simpl-1.4.0.vsix`.
2. В VSCode: `Extensions → ... → Install from VSIX...`.
3. Выберите файл.

### Из исходников

1. Откройте папку `vscode-simpl` в VSCode.
2. Нажмите `F5` — откроется новое окно с расширением.
3. Откройте любой файл `.spl`.

### Сборка `.vsix`

```
npm install -g @vscode/vsce
cd vscode-simpl
vsce package
```

Получите `simpl-1.4.0.vsix`.

## Пример

```lua
-- factorial.spl
let fact (n) =
    if n <= 1 then 1
    else n * fact(n - 1)
    end
end

echo. "5! =", fact(5)

use string (*)

let person = { name = "Аня", age = 12 }
echo. "Имя:", upper(person.name)

let ^total = 0
for i = 1, 10 do
    ^total = total + i
end
echo. "Сумма 1..10 =", total

let describe (x) =
    match x
        0 -> "ноль"
        n if n > 0 -> "положительное"
        _ -> "отрицательное"
    end
end
echo. describe(42)
```

## Цветовые схемы

Расширение использует **стандартные** TextMate-скоупы, поэтому подсветка работает с любой темой VSCode:

| Конструкция | Скоуп |
|---|---|
| `import` | `keyword.control.import.simpl` |
| Путь модуля в `import utils` | `entity.name.namespace.simpl` |
| Путь в кавычках в `import "с:/файл"` | `string.quoted.double.path.simpl` |
| `let`, `if`, `then`, `for`, `end` | `keyword.control.simpl` |
| `echo` | `keyword.other.echo.simpl` |
| `echo.` | `keyword.other.echo.newline.simpl` |
| `string`, `math`, `tuple`, `struct`, `iter`, `file`, `datetime` | `support.class.module.simpl` |
| `len`, `type`, `str`, `num`, `range`, `input` | `support.function.builtin.simpl` |
| Имя функции в `let name (...)` | `entity.name.function.simpl` |
| `true`, `false`, `nil` | `constant.language.simpl` |
| Числа | `constant.numeric.simpl` |
| Строки | `string.quoted.double.simpl` |
| `$(...)` в строке | `meta.interpolation.simpl` |
| Комментарии | `comment.line.double-dash.simpl` |
| `^` (мутабельность) | `keyword.operator.mutability.simpl` |
| `\` (лямбда) | `keyword.operator.lambda.simpl` |
| `++` | `keyword.operator.simpl` |
| `->` | `keyword.operator.simpl` |

## Известные ограничения

- Нет автодополнения (LSP пока не реализован).
- Нет перехода к определению.
- Нет подсветки имён, импортированных через `use`/`import` (`(*)` или `(names)`) отдельно от обычных переменных — они выглядят как идентификаторы.
- Нет перехода к определению по `import` (LSP пока не реализован).
- Нет проверки синтаксиса в реальном времени (только при запуске `simpl --check`).

## Лицензия

MIT