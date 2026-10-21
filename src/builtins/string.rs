use std::cell::RefCell;
use std::rc::Rc;

use crate::interpreter::{Interpreter, IResult};
use crate::messages as msg;
use crate::value::Value;

pub fn call(_interp: &mut Interpreter, name: &str, args: Vec<Value>) -> IResult<Value> {
    match name {
        "upper" => str_map1(args, "string.upper", |s| s.to_uppercase()),
        "lower" => str_map1(args, "string.lower", |s| s.to_lowercase()),
        "trim" => str_map1(args, "string.trim", |s| s.trim().to_string()),
        "rep" => call_rep(args),
        "sub" => call_sub(args),
        "find" => call_find(args),
        "split" => call_split(args),
        "join" => call_join(args),
        "starts_with" => call_starts_with(args, true),
        "ends_with" => call_starts_with(args, false),
        "chars" => call_chars(args),
        "from_tuple" => call_from_tuple(args),
        _ => Err(msg::bi_unknown(&format!("string.{}", name))),
    }
}

fn as_str(v: &Value, fname: &str) -> IResult<String> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        other => Err(msg::bi_bad_type(fname, "строку", other.type_name())),
    }
}

fn as_idx(v: &Value, fname: &str) -> IResult<usize> {
    match v {
        Value::Number(n) => {
            if *n < 0.0 {
                Err(msg::bi_bad_type(fname, "неотрицательное число", &format!("{}", n)))
            } else {
                Ok(*n as usize)
            }
        }
        other => Err(msg::bi_bad_type(fname, "число", other.type_name())),
    }
}

fn str_map1<F: Fn(&str) -> String>(args: Vec<Value>, fname: &str, f: F) -> IResult<Value> {
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    Ok(Value::Str(f(&s)))
}

fn call_rep(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.rep";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    let n = as_idx(&args[1], fname)?;
    Ok(Value::Str(s.repeat(n)))
}

fn call_sub(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.sub";
    if args.len() < 2 || args.len() > 3 {
        return Err(msg::bi_arity_range(fname, 2, 3, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    let start = as_idx(&args[1], fname)?;
    let chars: Vec<char> = s.chars().collect();
    let end = if args.len() == 3 {
        as_idx(&args[2], fname)?
    } else {
        chars.len()
    };
    if start > chars.len() {
        return Err(msg::bi_index_out_of_bounds(fname, start, chars.len()));
    }
    if end > chars.len() {
        return Err(msg::bi_index_out_of_bounds(fname, end, chars.len()));
    }
    if start > end {
        return Ok(Value::Str(String::new()));
    }
    let out: String = chars[start..end].iter().collect();
    Ok(Value::Str(out))
}

fn call_find(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.find";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    let sub = as_str(&args[1], fname)?;
    let chars: Vec<char> = s.chars().collect();
    let subc: Vec<char> = sub.chars().collect();
    if subc.is_empty() {
        return Ok(Value::Number(0.0));
    }
    if subc.len() > chars.len() {
        return Ok(Value::Number(-1.0));
    }
    for i in 0..=(chars.len() - subc.len()) {
        if chars[i..i + subc.len()] == subc[..] {
            return Ok(Value::Number(i as f64));
        }
    }
    Ok(Value::Number(-1.0))
}

fn call_split(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.split";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    let sep = as_str(&args[1], fname)?;
    if sep.is_empty() {
        return Err(msg::bi_string_empty_sep());
    }
    let parts: Vec<Value> = s.split(&sep).map(|p| Value::Str(p.to_string())).collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(parts))))
}

fn call_join(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.join";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let t = match &args[0] {
        Value::Tuple(t) => t.borrow().clone(),
        other => return Err(msg::bi_bad_type(fname, "кортеж", other.type_name())),
    };
    let sep = as_str(&args[1], fname)?;
    let mut parts = Vec::with_capacity(t.len());
    for v in t {
        parts.push(v.display());
    }
    Ok(Value::Str(parts.join(&sep)))
}

fn call_starts_with(args: Vec<Value>, starts: bool) -> IResult<Value> {
    let fname = if starts { "string.starts_with" } else { "string.ends_with" };
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    let p = as_str(&args[1], fname)?;
    Ok(Value::Bool(if starts { s.starts_with(&p) } else { s.ends_with(&p) }))
}

fn call_chars(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.chars";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    let v: Vec<Value> = s.chars().map(|c| Value::Str(c.to_string())).collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(v))))
}

fn call_from_tuple(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.from_tuple";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let t = match &args[0] {
        Value::Tuple(t) => t.borrow().clone(),
        other => return Err(msg::bi_bad_type(fname, "кортеж", other.type_name())),
    };
    let mut s = String::new();
    for v in t {
        s.push_str(&v.display());
    }
    Ok(Value::Str(s))
}