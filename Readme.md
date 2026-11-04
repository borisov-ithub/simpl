# Simpl

Учебный язык программирования с Lua-подобным синтаксисом, строгой мутабельностью и выражениями вместо инструкций. Написан на Rust.

## Зачем

Simpl создан для обучения программированию. Он убирает неявные механизмы (глобальные переменные по умолчанию, метатаблицы, `nil` вместо ошибок) и делает поведение программы предсказуемым. Каждая конструкция — выражение, у которого есть значение.

## Сборка

```bash
cargo build --release
```
## Запуск
```bash
simpl program.spl
```
## Пример
```
-- factorial.spl
let fact (n) =
    if n <= 1 then 1
    else n * fact(n - 1)
    end
end

echo. "5! =", fact(5)

let person = { name = "Аня", age = 12 }
echo. "Имя:", person.name

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
