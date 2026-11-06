pub mod ast;
pub mod builtins;
pub mod checker;
pub mod env;
pub mod interpreter;
pub mod lexer;
pub mod messages;
pub mod parser;
pub mod repl;
pub mod types;
pub mod value;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub fn run_source(src: &str) -> Result<(), String> {
    run_source_with_path(src, None, vec![])
}

pub fn run_source_with_path(
    src: &str,
    current_file: Option<PathBuf>,
    argv: Vec<String>,
) -> Result<(), String> {
    let mut lexer = lexer::Lexer::new(src);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;

    let mut checker = checker::Checker::new()
        .with_file(current_file.clone())
        .with_type_hints(src);
    checker.check_program(&program)?;

    let mut interp = interpreter::Interpreter::new();
    interp.echo_to_stdout = true;
    interp.current_file = current_file;
    {
        let mut g = interp.globals.borrow_mut();
        builtins::register(&mut g.vars);
        let tuple: Vec<value::Value> = argv
            .iter()
            .map(|s| value::Value::Str(s.clone()))
            .collect();
        g.define(
            "argv",
            value::Value::Tuple(Rc::new(RefCell::new(tuple))),
            false,
        );
    }
    interp.run(&program)?;
    Ok(())
}

pub fn run_source_capture(src: &str) -> Result<String, String> {
    run_source_capture_with_path(src, None, vec![])
}

pub fn run_source_capture_with_argv(
    src: &str,
    argv: Vec<String>,
) -> Result<String, String> {
    run_source_capture_with_path(src, None, argv)
}

pub fn run_source_capture_with_path(
    src: &str,
    current_file: Option<PathBuf>,
    argv: Vec<String>,
) -> Result<String, String> {
    let mut lexer = lexer::Lexer::new(src);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;

    let mut checker = checker::Checker::new()
        .with_file(current_file.clone())
        .with_type_hints(src);
    checker.check_program(&program)?;

    let mut interp = interpreter::Interpreter::new();
    interp.current_file = current_file;
    {
        let mut g = interp.globals.borrow_mut();
        builtins::register(&mut g.vars);
        let tuple: Vec<value::Value> = argv
            .iter()
            .map(|s| value::Value::Str(s.clone()))
            .collect();
        g.define(
            "argv",
            value::Value::Tuple(Rc::new(RefCell::new(tuple))),
            false,
        );
    }
    interp.run(&program)?;
    Ok(interp.take_output())
}

pub fn check_source(src: &str) -> Result<(), String> {
    let mut lexer = lexer::Lexer::new(src);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;
    let mut checker = checker::Checker::new().with_type_hints(src);
    checker.check_program(&program)?;
    Ok(())
}

