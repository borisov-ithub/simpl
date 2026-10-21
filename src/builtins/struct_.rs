use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::{Interpreter, IResult};
use crate::messages as msg;
use crate::value::Value;

pub fn call(_interp: &mut Interpreter, name: &str, args: Vec<Value>) -> IResult<Value> {
    match name {
        "keys" => call_keys(args),
        "values" => call_values(args),
        "entries" => call_entries(args),
        "has" => call_has(args),
        "merge" => call_merge(args),
        "remove" => call_remove(args),
        "from_tuple" => call_from_tuple(args),
        "from_pairs" => call_from_pairs(args),
        "zip" => call_zip(args),
        _ => Err(msg::bi_unknown(&format!("struct.{}", name))),
    }
}

fn as_struct(v: &Value, fname: &str) -> IResult<Rc<RefCell<HashMap<String, Value>>>> {
    match v {
        Value::Struct(s) => Ok(s.clone()),
        other => Err(msg::bi_bad_type(fname, "struct", other.type_name())),
    }
}

/// Сортировка ключей: числовые — как числа, иначе — лексикографически.
fn sorted_keys(map: &HashMap<String, Value>) -> Vec<String> {
    let mut keys: Vec<String> = map.keys().cloned().collect();
    keys.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) {
        (Ok(x), Ok(y)) => x.cmp(&y),
        _ => a.cmp(b),
    });
    keys
}

fn call_keys(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.keys";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let b = s.borrow();
    let keys = sorted_keys(&b);
    let v: Vec<Value> = keys.into_iter().map(Value::Str).collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(v))))
}

fn call_values(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.values";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let b = s.borrow();
    let keys = sorted_keys(&b);
    let v: Vec<Value> = keys.into_iter().map(|k| b[&k].clone()).collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(v))))
}

fn call_entries(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.entries";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let b = s.borrow();
    let keys = sorted_keys(&b);
    let v: Vec<Value> = keys
        .into_iter()
        .map(|k| {
            let pair = vec![Value::Str(k.clone()), b[&k].clone()];
            Value::Tuple(Rc::new(RefCell::new(pair)))
        })
        .collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(v))))
}

fn call_has(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.has";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let k = match &args[1] {
        Value::Str(s) => s.clone(),
        other => return Err(msg::bi_bad_type(fname, "ключ-строку", other.type_name())),
    };
    let contains = s.borrow().contains_key(&k);
    Ok(Value::Bool(contains))
}

fn call_merge(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.merge";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let a = as_struct(&args[0], fname)?;
    let b = as_struct(&args[1], fname)?;
    let mut out = a.borrow().clone();
    for (k, v) in b.borrow().iter() {
        out.insert(k.clone(), v.clone());
    }
    Ok(Value::Struct(Rc::new(RefCell::new(out))))
}

fn call_remove(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.remove";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let k = match &args[1] {
        Value::Str(s) => s.clone(),
        other => return Err(msg::bi_bad_type(fname, "ключ-строку", other.type_name())),
    };
    let mut out = s.borrow().clone();
    out.remove(&k);
    Ok(Value::Struct(Rc::new(RefCell::new(out))))
}

fn call_from_tuple(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.from_tuple";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let t = match &args[0] {
        Value::Tuple(t) => t.borrow().clone(),
        other => return Err(msg::bi_bad_type(fname, "кортеж", other.type_name())),
    };
    let mut map = HashMap::new();
    for (i, v) in t.into_iter().enumerate() {
        map.insert(i.to_string(), v);
    }
    Ok(Value::Struct(Rc::new(RefCell::new(map))))
}

fn call_from_pairs(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.from_pairs";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let t = match &args[0] {
        Value::Tuple(t) => t.borrow().clone(),
        other => return Err(msg::bi_bad_type(fname, "кортеж", other.type_name())),
    };
    let mut map = HashMap::new();
    for pair in t {
        let items: Vec<Value> = match &pair {
            Value::Tuple(p) => p.borrow().clone(),
            other => return Err(msg::bi_struct_from_pairs_bad_elem(fname, other.type_name())),
        };
        if items.len() != 2 {
            return Err(msg::bi_struct_from_pairs_bad_elem(
                fname,
                &format!("кортеж длины {}", items.len()),
            ));
        }
        let k = items[0].display();
        let v = items[1].clone();
        map.insert(k, v);
    }
    Ok(Value::Struct(Rc::new(RefCell::new(map))))
}

fn call_zip(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.zip";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let keys = match &args[0] {
        Value::Tuple(t) => t.borrow().clone(),
        other => return Err(msg::bi_bad_type(fname, "кортеж", other.type_name())),
    };
    let vals = match &args[1] {
        Value::Tuple(t) => t.borrow().clone(),
        other => return Err(msg::bi_bad_type(fname, "кортеж", other.type_name())),
    };
    if keys.len() != vals.len() {
        return Err(msg::bi_struct_zip_len_mismatch(fname, keys.len(), vals.len()));
    }
    let mut map = HashMap::new();
    for (k, v) in keys.into_iter().zip(vals.into_iter()) {
        map.insert(k.display(), v);
    }
    Ok(Value::Struct(Rc::new(RefCell::new(map))))
}