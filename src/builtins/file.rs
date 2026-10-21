use std::fs;
use std::io::Write;

use crate::interpreter::{Interpreter, IResult};
use crate::messages as msg;
use crate::value::Value;

pub fn call(_interp: &mut Interpreter, name: &str, args: Vec<Value>) -> IResult<Value> {
    match name {
        "read" => call_read(args),
        "write" => call_write(args, false),
        "append" => call_write(args, true),
        "exists" => call_exists(args),
        _ => Err(msg::bi_unknown(&format!("file.{}", name))),
    }
}

fn as_str(v: &Value, fname: &str) -> IResult<String> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        other => Err(msg::bi_bad_type(fname, "строку", other.type_name())),
    }
}

fn call_read(args: Vec<Value>) -> IResult<Value> {
    let fname = "file.read";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let path = as_str(&args[0], fname)?;
    let content = fs::read_to_string(&path)
        .map_err(|e| msg::bi_file_read_error(&path, &e))?;
    Ok(Value::Str(content))
}

fn call_write(args: Vec<Value>, append: bool) -> IResult<Value> {
    let fname = if append { "file.append" } else { "file.write" };
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let path = as_str(&args[0], fname)?;
    let content = as_str(&args[1], fname)?;
    if append {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| msg::bi_file_append_error(&path, &e))?;
        f.write_all(content.as_bytes())
            .map_err(|e| msg::bi_file_append_error(&path, &e))?;
    } else {
        fs::write(&path, content.as_bytes())
            .map_err(|e| msg::bi_file_write_error(&path, &e))?;
    }
    Ok(Value::Nil)
}

fn call_exists(args: Vec<Value>) -> IResult<Value> {
    let fname = "file.exists";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let path = as_str(&args[0], fname)?;
    Ok(Value::Bool(std::path::Path::new(&path).exists()))
}