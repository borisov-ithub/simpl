use std::cell::RefCell;
use std::rc::Rc;

use crate::interpreter::{Interpreter, IResult};
use crate::messages as msg;
use crate::value::Value;

pub fn call(_interp: &mut Interpreter, name: &str, args: Vec<Value>) -> IResult<Value> {
    match name {
        "get" => call_get(args),
        "slice" => call_slice(args),
        "concat" => call_concat(args),
        "push" => call_push(args),
        "reverse" => call_reverse(args),
        "index_of" => call_index_of(args),
        "contains" => call_contains(args),
        "sort" => call_sort(args),
        "zip" => call_zip(args),
        _ => Err(msg::bi_unknown(&format!("tuple.{}", name))),
    }
}

fn as_tuple(v: &Value, fname: &str) -> IResult<Vec<Value>> {
    match v {
        Value::Tuple(t) => Ok(t.borrow().clone()),
        other => Err(msg::bi_bad_type(fname, "кортеж", other.type_name())),
    }
}

fn as_idx(v: &Value, fname: &str) -> IResult<usize> {
    match v {
        Value::Number(n) if *n >= 0.0 => Ok(*n as usize),
        other => Err(msg::bi_bad_type(fname, "неотрицательное число", other.type_name())),
    }
}

fn call_get(args: Vec<Value>) -> IResult<Value> {
    let fname = "tuple.get";
    if args.len() < 2 || args.len() > 3 {
        return Err(msg::bi_arity_range(fname, 2, 3, args.len()));
    }
    let t = as_tuple(&args[0], fname)?;
    let i = as_idx(&args[1], fname)?;
    if i < t.len() {
        Ok(t[i].clone())
    } else if args.len() == 3 {
        Ok(args[2].clone())
    } else {
        Err(msg::bi_index_out_of_bounds(fname, i, t.len()))
    }
}

fn call_slice(args: Vec<Value>) -> IResult<Value> {
    let fname = "tuple.slice";
    if args.len() < 2 || args.len() > 3 {
        return Err(msg::bi_arity_range(fname, 2, 3, args.len()));
    }
    let t = as_tuple(&args[0], fname)?;
    let start = as_idx(&args[1], fname)?;
    let end = if args.len() == 3 { as_idx(&args[2], fname)? } else { t.len() };
    if start > t.len() {
        return Err(msg::bi_index_out_of_bounds(fname, start, t.len()));
    }
    if end > t.len() {
        return Err(msg::bi_index_out_of_bounds(fname, end, t.len()));
    }
    if start > end {
        return Ok(Value::Tuple(Rc::new(RefCell::new(vec![]))));
    }
    let out: Vec<Value> = t[start..end].to_vec();
    Ok(Value::Tuple(Rc::new(RefCell::new(out))))
}

fn call_concat(args: Vec<Value>) -> IResult<Value> {
    let fname = "tuple.concat";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let mut a = as_tuple(&args[0], fname)?;
    let b = as_tuple(&args[1], fname)?;
    a.extend(b);
    Ok(Value::Tuple(Rc::new(RefCell::new(a))))
}

fn call_push(args: Vec<Value>) -> IResult<Value> {
    let fname = "tuple.push";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let mut t = as_tuple(&args[0], fname)?;
    t.push(args[1].clone());
    Ok(Value::Tuple(Rc::new(RefCell::new(t))))
}

fn call_reverse(args: Vec<Value>) -> IResult<Value> {
    let fname = "tuple.reverse";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let mut t = as_tuple(&args[0], fname)?;
    t.reverse();
    Ok(Value::Tuple(Rc::new(RefCell::new(t))))
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Tuple(x), Value::Tuple(y)) => {
            let xb = x.borrow();
            let yb = y.borrow();
            xb.len() == yb.len() && xb.iter().zip(yb.iter()).all(|(a, b)| values_equal(a, b))
        }
        _ => false,
    }
}

fn call_index_of(args: Vec<Value>) -> IResult<Value> {
    let fname = "tuple.index_of";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let t = as_tuple(&args[0], fname)?;
    for (i, v) in t.iter().enumerate() {
        if values_equal(v, &args[1]) {
            return Ok(Value::Number(i as f64));
        }
    }
    Ok(Value::Number(-1.0))
}

fn call_contains(args: Vec<Value>) -> IResult<Value> {
    let fname = "tuple.contains";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let t = as_tuple(&args[0], fname)?;
    Ok(Value::Bool(t.iter().any(|v| values_equal(v, &args[1]))))
}

fn call_sort(args: Vec<Value>) -> IResult<Value> {
    let fname = "tuple.sort";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let mut t = as_tuple(&args[0], fname)?;
    let mut nums: Vec<f64> = Vec::with_capacity(t.len());
    for v in &t {
        match v {
            Value::Number(n) => nums.push(*n),
            other => return Err(msg::bi_bad_type(fname, "кортеж чисел", other.type_name())),
        }
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    t = nums.into_iter().map(Value::Number).collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(t))))
}

fn call_zip(args: Vec<Value>) -> IResult<Value> {
    let fname = "tuple.zip";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let a = as_tuple(&args[0], fname)?;
    let b = as_tuple(&args[1], fname)?;
    if a.len() != b.len() {
        return Err(msg::bi_tuple_zip_len_mismatch(fname, a.len(), b.len()));
    }
    let out: Vec<Value> = a
        .into_iter()
        .zip(b.into_iter())
        .map(|(x, y)| Value::Tuple(Rc::new(RefCell::new(vec![x, y]))))
        .collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(out))))
}