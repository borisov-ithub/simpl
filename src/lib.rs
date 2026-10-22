pub mod ast;
pub mod builtins;
pub mod env;
pub mod interpreter;
pub mod lexer;
pub mod messages;
pub mod parser;
pub mod value;

pub fn run_source(src: &str) -> Result<(), String> {
    let out = run_source_capture(src)?;
    print!("{}", out);
    Ok(())
}

pub fn run_source_capture(src: &str) -> Result<String, String> {
    let mut lexer = lexer::Lexer::new(src);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;
    let mut interp = interpreter::Interpreter::new();
    {
        let mut g = interp.globals.borrow_mut();
        builtins::register(&mut g.vars);
    }
    interp.run(&program)?;
    Ok(interp.take_output())
}

pub fn check_source(src: &str) -> Result<(), String> {
    let mut lexer = lexer::Lexer::new(src);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    let _program = parser.parse_program()?;
    Ok(())
}