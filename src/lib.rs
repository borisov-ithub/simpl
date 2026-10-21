pub mod ast;
pub mod env;
pub mod interpreter;
pub mod lexer;
pub mod messages;
pub mod parser;
pub mod value;

/// Запускает исходник, вывод `echo`/`echo.` пишется в stdout.
pub fn run_source(src: &str) -> Result<(), String> {
    let mut lexer = lexer::Lexer::new(src);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;
    let mut interp = interpreter::Interpreter::new();
    register_builtins(&interp);
    interp.run(&program)?;
    print!("{}", interp.take_output());
    Ok(())
}

/// Запускает исходник и возвращает весь вывод как строку.
pub fn run_source_capture(src: &str) -> Result<String, String> {
    let mut lexer = lexer::Lexer::new(src);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;
    let mut interp = interpreter::Interpreter::new();
    register_builtins(&interp);
    interp.run(&program)?;
    Ok(interp.take_output())
}

fn register_builtins(interp: &interpreter::Interpreter) {
    let mut g = interp.globals.borrow_mut();
    for name in ["len", "type", "str", "num", "range"] {
        g.define(name, value::Value::Builtin(name), false);
    }
}