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
        "ltrim" => str_map1(args, "string.ltrim", |s| s.trim_start().to_string()),
        "rtrim" => str_map1(args, "string.rtrim", |s| s.trim_end().to_string()),
        "capitalize" => call_capitalize(args),
        "swap_case" => str_map1(args, "string.swap_case", swap_case),
        "title" => str_map1(args, "string.title", title_case),
        "rep" => call_rep(args),
        "sub" => call_sub(args),
        "find" => call_find(args),
        "split" => call_split(args),
        "join" => call_join(args),
        "starts_with" => call_starts_with(args, true),
        "ends_with" => call_starts_with(args, false),
        "chars" => call_chars(args),
        "from_tuple" => call_from_tuple(args),
        "count" => call_count(args),
        "replace" => call_replace(args),
        "is_upper" => call_is_pred(args, "string.is_upper", |s| is_upper(s)),
        "is_lower" => call_is_pred(args, "string.is_lower", |s| is_lower(s)),
        "is_alnum" => call_is_pred(args, "string.is_alnum", |s| is_alnum(s)),
        "is_alpha" => call_is_pred(args, "string.is_alpha", |s| is_alpha(s)),
        "is_decimal" => call_is_pred(args, "string.is_decimal", |s| is_decimal(s)),
        "is_numeric" => call_is_pred(args, "string.is_numeric", |s| is_numeric(s)),
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

fn swap_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_uppercase() {
                c.to_lowercase().next().unwrap_or(c)
            } else if c.is_lowercase() {
                c.to_uppercase().next().unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

fn title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_is_space = true;
    for c in s.chars() {
        if prev_is_space {
            for u in c.to_uppercase() {
                out.push(u);
            }
        } else {
            for l in c.to_lowercase() {
                out.push(l);
            }
        }
        prev_is_space = c.is_whitespace();
    }
    out
}

fn is_upper(s: &str) -> bool {
    let mut has_letter = false;
    for c in s.chars() {
        if c.is_lowercase() {
            return false;
        }
        if c.is_uppercase() {
            has_letter = true;
        }
    }
    has_letter
}

fn is_lower(s: &str) -> bool {
    let mut has_letter = false;
    for c in s.chars() {
        if c.is_uppercase() {
            return false;
        }
        if c.is_lowercase() {
            has_letter = true;
        }
    }
    has_letter
}

fn is_alnum(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric())
}

fn is_alpha(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphabetic())
}

fn is_numeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn is_decimal(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut dot_pos = None;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'.' {
            if dot_pos.is_some() {
                return false;
            }
            dot_pos = Some(i);
        } else if !b.is_ascii_digit() {
            return false;
        }
    }
    match dot_pos {
        Some(p) => p > 0 && p < bytes.len() - 1,
        None => false,
    }
}

fn call_is_pred<F: Fn(&str) -> bool>(args: Vec<Value>, fname: &str, f: F) -> IResult<Value> {
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    Ok(Value::Bool(f(&s)))
}

fn call_capitalize(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.capitalize";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    let mut chars = s.chars();
    let out = match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut out: String = first.to_uppercase().collect();
            for c in chars {
                for l in c.to_lowercase() {
                    out.push(l);
                }
            }
            out
        }
    };
    Ok(Value::Str(out))
}

fn call_count(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.count";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    let sub = as_str(&args[1], fname)?;
    if sub.is_empty() {
        return Err(msg::bi_string_count_empty_sub());
    }
    let hay: Vec<char> = s.chars().collect();
    let needle: Vec<char> = sub.chars().collect();
    if needle.len() > hay.len() {
        return Ok(Value::Number(0.0));
    }
    let mut count = 0usize;
    let mut i = 0usize;
    while i + needle.len() <= hay.len() {
        if hay[i..i + needle.len()] == needle[..] {
            count += 1;
            i += 1;
        } else {
            i += 1;
        }
    }
    Ok(Value::Number(count as f64))
}

fn call_replace(args: Vec<Value>) -> IResult<Value> {
    let fname = "string.replace";
    if args.len() != 3 {
        return Err(msg::bi_arity(fname, 3, args.len()));
    }
    let s = as_str(&args[0], fname)?;
    let from = as_str(&args[1], fname)?;
    let to = as_str(&args[2], fname)?;
    if from.is_empty() {
        // не делаем ничего
        let out = vec![Value::Str(s), Value::Number(0.0)];
        return Ok(Value::Tuple(Rc::new(RefCell::new(out))));
    }
    let count = s.matches(&from).count();
    let new_s = s.replace(&from, &to);
    let out = vec![Value::Str(new_s), Value::Number(count as f64)];
    Ok(Value::Tuple(Rc::new(RefCell::new(out))))
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