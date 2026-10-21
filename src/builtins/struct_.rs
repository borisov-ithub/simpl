use std::cell::RefCell;
use std::rc::Rc;

use crate::interpreter::{key_to_value, to_key_string, Interpreter, IResult};
use crate::messages as msg;
use crate::value::{OrderedKey, StructMap, Value};

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

fn as_struct(v: &Value, fname: &str) -> IResult<Rc<RefCell<StructMap>>> {
    match v {
        Value::Struct(s) => Ok(s.clone()),
        other => Err(msg::bi_bad_type(fname, "struct", other.type_name())),
    }
}

fn call_keys(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.keys";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let b = s.borrow();
    let v: Vec<Value> = b.keys().map(key_to_value).collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(v))))
}

fn call_values(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.values";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let b = s.borrow();
    let v: Vec<Value> = b.values().cloned().collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(v))))
}

fn call_entries(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.entries";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let b = s.borrow();
    let v: Vec<Value> = b
        .iter()
        .map(|(k, val)| {
            let pair = vec![key_to_value(k), val.clone()];
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
    let key = OrderedKey(to_key_string(&args[1]));
    let contains = s.borrow().contains_key(&key);
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
    let key = OrderedKey(to_key_string(&args[1]));
    let mut out = s.borrow().clone();
    out.remove(&key);
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
    let mut map = StructMap::new();
    for (i, v) in t.into_iter().enumerate() {
        map.insert(OrderedKey(i.to_string()), v);
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
    let mut map = StructMap::new();
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
        let k = OrderedKey(to_key_string(&items[0]));
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
    let mut map = StructMap::new();
    for (k, v) in keys.into_iter().zip(vals.into_iter()) {
        map.insert(OrderedKey(to_key_string(&k)), v);
    }
    Ok(Value::Struct(Rc::new(RefCell::new(map))))
}