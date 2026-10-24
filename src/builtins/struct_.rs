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
        "get" => call_get(args),
        "push" => call_push(args),
        "pop" => call_pop(args),
        "unshift" => call_unshift(args),
        "shift" => call_shift(args),
        "insert_at" => call_insert_at(args),
        "remove_at" => call_remove_at(args),
        _ => Err(msg::bi_unknown(&format!("struct.{}", name))),
    }
}

fn as_struct(v: &Value, fname: &str) -> IResult<Rc<RefCell<StructMap>>> {
    match v {
        Value::Struct(s) => Ok(s.clone()),
        other => Err(msg::bi_bad_type(fname, "struct", other.type_name())),
    }
}

/// Проверяет, что все ключи — числа. Возвращает отсортированный список ключей.
fn numeric_keys(s: &StructMap, fname: &str) -> IResult<Vec<i64>> {
    let mut keys: Vec<i64> = Vec::with_capacity(s.len());
    for k in s.keys() {
        match k.0.parse::<i64>() {
            Ok(n) => keys.push(n),
            Err(_) => return Err(msg::bi_struct_not_array(fname)),
        }
    }
    keys.sort();
    Ok(keys)
}

fn call_keys(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.keys";
    if args.len() != 1 { return Err(msg::bi_arity(fname, 1, args.len())); }
    let s = as_struct(&args[0], fname)?;
    let b = s.borrow();
    let v: Vec<Value> = b.keys().map(key_to_value).collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(v))))
}

fn call_values(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.values";
    if args.len() != 1 { return Err(msg::bi_arity(fname, 1, args.len())); }
    let s = as_struct(&args[0], fname)?;
    let b = s.borrow();
    let v: Vec<Value> = b.values().cloned().collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(v))))
}

fn call_entries(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.entries";
    if args.len() != 1 { return Err(msg::bi_arity(fname, 1, args.len())); }
    let s = as_struct(&args[0], fname)?;
    let b = s.borrow();
    let v: Vec<Value> = b.iter().map(|(k, val)| {
        let pair = vec![key_to_value(k), val.clone()];
        Value::Tuple(Rc::new(RefCell::new(pair)))
    }).collect();
    Ok(Value::Tuple(Rc::new(RefCell::new(v))))
}

fn call_has(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.has";
    if args.len() != 2 { return Err(msg::bi_arity(fname, 2, args.len())); }
    let s = as_struct(&args[0], fname)?;
    let key = OrderedKey(to_key_string(&args[1]));
    let contains = s.borrow().contains_key(&key);
    Ok(Value::Bool(contains))
}

fn call_merge(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.merge";
    if args.len() != 2 { return Err(msg::bi_arity(fname, 2, args.len())); }
    let a = as_struct(&args[0], fname)?;
    let b = as_struct(&args[1], fname)?;
    // мутируем a
    let mut a_mut = a.borrow_mut();
    for (k, v) in b.borrow().iter() {
        a_mut.insert(k.clone(), v.clone());
    }
    Ok(Value::Nil)
}

fn call_remove(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.remove";
    if args.len() != 2 { return Err(msg::bi_arity(fname, 2, args.len())); }
    let s = as_struct(&args[0], fname)?;
    let key = OrderedKey(to_key_string(&args[1]));
    s.borrow_mut().remove(&key);
    Ok(Value::Nil)
}

fn call_from_tuple(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.from_tuple";
    if args.len() != 1 { return Err(msg::bi_arity(fname, 1, args.len())); }
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
    if args.len() != 1 { return Err(msg::bi_arity(fname, 1, args.len())); }
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
    if args.len() != 2 { return Err(msg::bi_arity(fname, 2, args.len())); }
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

fn call_get(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.get";
    if args.len() < 2 || args.len() > 3 {
        return Err(msg::bi_arity_range(fname, 2, 3, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let key = OrderedKey(to_key_string(&args[1]));
    let b = s.borrow();
    if let Some(v) = b.get(&key) {
        Ok(v.clone())
    } else if args.len() == 3 {
        Ok(args[2].clone())
    } else {
        Err(msg::rt_field_missing(&key.0))
    }
}

fn call_push(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.push";
    if args.len() != 2 { return Err(msg::bi_arity(fname, 2, args.len())); }
    let s = as_struct(&args[0], fname)?;
    let keys = numeric_keys(&s.borrow(), fname)?;
    let next = keys.last().copied().unwrap_or(-1) + 1;
    s.borrow_mut().insert(OrderedKey(next.to_string()), args[1].clone());
    Ok(Value::Nil)
}

fn call_pop(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.pop";
    if args.len() != 1 { return Err(msg::bi_arity(fname, 1, args.len())); }
    let s = as_struct(&args[0], fname)?;
    let keys = numeric_keys(&s.borrow(), fname)?;
    let last = match keys.last() {
        Some(k) => *k,
        None => return Err(msg::bi_struct_empty(fname)),
    };
    let removed = s.borrow_mut().remove(&OrderedKey(last.to_string()));
    Ok(removed.unwrap_or(Value::Nil))
}

fn call_unshift(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.unshift";
    if args.len() != 2 { return Err(msg::bi_arity(fname, 2, args.len())); }
    let s = as_struct(&args[0], fname)?;
    let keys = numeric_keys(&s.borrow(), fname)?;
    // сдвигаем все ключи на 1 (в порядке убывания, чтобы не перезаписать)
    let mut guard = s.borrow_mut();
    for k in keys.iter().rev() {
        let v = guard.remove(&OrderedKey(k.to_string())).unwrap();
        guard.insert(OrderedKey((k + 1).to_string()), v);
    }
    guard.insert(OrderedKey("0".to_string()), args[1].clone());
    Ok(Value::Nil)
}

fn call_shift(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.shift";
    if args.len() != 1 { return Err(msg::bi_arity(fname, 1, args.len())); }
    let s = as_struct(&args[0], fname)?;
    let keys = numeric_keys(&s.borrow(), fname)?;
    if keys.is_empty() {
        return Err(msg::bi_struct_empty(fname));
    }
    let first = s.borrow_mut().remove(&OrderedKey(keys[0].to_string())).unwrap();
    // сдвигаем остальные на -1
    let mut guard = s.borrow_mut();
    for k in &keys[1..] {
        let v = guard.remove(&OrderedKey(k.to_string())).unwrap();
        guard.insert(OrderedKey((k - 1).to_string()), v);
    }
    Ok(first)
}

fn call_insert_at(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.insert_at";
    if args.len() != 3 {
        return Err(msg::bi_arity(fname, 3, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let keys = numeric_keys(&s.borrow(), fname)?;
    let index = match &args[1] {
        Value::Number(n) if *n >= 0.0 && n.fract() == 0.0 => *n as i64,
        other => {
            return Err(msg::bi_bad_type(
                fname,
                "неотрицательный целый индекс",
                other.type_name(),
            ))
        }
    };
    if index > keys.len() as i64 {
        return Err(msg::bi_index_out_of_bounds(fname, index as usize, keys.len()));
    }
    let mut guard = s.borrow_mut();
    for k in keys.iter().rev() {
        if *k >= index {
            let v = guard.remove(&OrderedKey(k.to_string())).unwrap();
            guard.insert(OrderedKey((k + 1).to_string()), v);
        }
    }
    guard.insert(OrderedKey(index.to_string()), args[2].clone());
    Ok(Value::Nil)
}

fn call_remove_at(args: Vec<Value>) -> IResult<Value> {
    let fname = "struct.remove_at";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let s = as_struct(&args[0], fname)?;
    let keys = numeric_keys(&s.borrow(), fname)?;
    if keys.is_empty() {
        return Err(msg::bi_struct_empty(fname));
    }
    let index = match &args[1] {
        Value::Number(n) if *n >= 0.0 && n.fract() == 0.0 => *n as i64,
        other => {
            return Err(msg::bi_bad_type(
                fname,
                "неотрицательный целый индекс",
                other.type_name(),
            ))
        }
    };
    if index >= keys.len() as i64 {
        return Err(msg::bi_index_out_of_bounds(fname, index as usize, keys.len()));
    }
    let mut guard = s.borrow_mut();
    let removed = guard.remove(&OrderedKey(index.to_string())).unwrap();
    for k in keys.iter() {
        if *k > index {
            let v = guard.remove(&OrderedKey(k.to_string())).unwrap();
            guard.insert(OrderedKey((k - 1).to_string()), v);
        }
    }
    Ok(removed)
}