use std::path::PathBuf;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::ast::Block;
use crate::builtins;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::messages as msg;
use crate::parser::Parser;
use crate::value::Value;

pub fn run() -> Result<(), String> {
    let mut interp = new_interp();
    let mut rl = DefaultEditor::new().map_err(|e| e.to_string())?;
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { ">>> " } else { "... " };
        let line = match rl.readline(prompt) {
            Ok(l) => l,
            Err(ReadlineError::Interrupted) => {
                buffer.clear();
                continue;
            }
            Err(ReadlineError::Eof) => break,
            Err(e) => return Err(e.to_string()),
        };
        let _ = rl.add_history_entry(line.as_str());

        // Команды — только когда буфер пуст
        if buffer.is_empty() && line.starts_with(':') {
            match handle_command(&line, &mut interp) {
                CommandResult::Continue => continue,
                CommandResult::Quit => break,
            }
        }

        if !buffer.is_empty() { buffer.push('\n'); }
        buffer.push_str(&line);

        match try_parse(&buffer) {
            ParseResult::Ok(program) => {
                buffer.clear();
                interp.out.borrow_mut().clear();
                let result = interp.run(&program);
                let out = interp.take_output();
                print!("{}", out);
                match result {
                    Ok(val) => {
                        println!("{}", val.display());
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
            ParseResult::Incomplete => {
                // продолжаем накапливать
            }
            ParseResult::Error(e) => {
                buffer.clear();
                eprintln!("{}", e);
            }
        }
    }

    Ok(())
}

fn new_interp() -> Interpreter {
    let interp = Interpreter::new();
    {
        let mut g = interp.globals.borrow_mut();
        builtins::register(&mut g.vars);
        use std::cell::RefCell;
        use std::rc::Rc;
        g.define(
            "argv",
            crate::value::Value::Tuple(Rc::new(RefCell::new(vec![]))),
            false,
        );
    }
    interp
}

enum CommandResult {
    Continue,
    Quit,
}

fn handle_command(line: &str, interp: &mut Interpreter) -> CommandResult {
    let trimmed = line.trim();
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let cmd = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("").trim();

    match cmd {
        ":q" | ":quit" => CommandResult::Quit,
        ":h" | ":help" => {
            println!("{}", msg::repl_help());
            CommandResult::Continue
        }
        ":reset" => {
            *interp = new_interp();
            println!("{}", msg::repl_reset_ok());
            CommandResult::Continue
        }
        ":load" => {
            if rest.is_empty() {
                eprintln!("{}", msg::repl_load_usage());
                return CommandResult::Continue;
            }
            let path = rest.trim_matches('"');
            match load_file(interp, path) {
                Ok(()) => {}
                Err(e) => eprintln!("{}", e),
            }
            CommandResult::Continue
        }
        other => {
            eprintln!("{}", msg::repl_unknown_command(other));
            CommandResult::Continue
        }
    }
}

fn load_file(interp: &mut Interpreter, path: &str) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    let full = path_buf
        .canonicalize()
        .map_err(|e| msg::repl_load_error(path, &e))?;
    let src = std::fs::read_to_string(&full)
        .map_err(|e| msg::repl_load_error(&full.display().to_string(), &e))?;

    let mut lexer = Lexer::new(&src);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    // Сохраняем текущий файл, ставим загружаемый
    let old_file = interp.current_file.clone();
    interp.current_file = Some(full.clone());

    interp.out.borrow_mut().clear();
    let result = run_in_globals(interp, &program);
    let out = interp.take_output();
    print!("{}", out);

    interp.current_file = old_file;

    match result {
        Ok(val) => {
            if !matches!(val, Value::Nil) {
                println!("{}", val.display());
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn run_in_globals(interp: &mut Interpreter, program: &Block) -> Result<Value, String> {
    let env = interp.globals.clone();
    interp.run_in_env(program, &env)
}

enum ParseResult {
    Ok(Block),
    Incomplete,
    Error(String),
}

fn try_parse(src: &str) -> ParseResult {
    let mut lexer = Lexer::new(src);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => return ParseResult::Error(e),
    };
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => ParseResult::Ok(program),
        Err(e) => {
            if e.contains("начатой в строке") && e.contains("достигнут конец") {
                ParseResult::Incomplete
            } else {
                ParseResult::Error(e)
            }
        }
    }
}