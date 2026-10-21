use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::{Interpreter, IResult};
use crate::messages as msg;
use crate::value::Value;

pub fn call(interp: &mut Interpreter, name: &str, args: Vec<Value>) -> IResult<Value> {
    match name {
        "map" => call_map(interp, args),
        "filter" => call_filter(interp, args),
        "reduce" => call_reduce(interp, args),
        _ => Err(msg::bi_unknown(&format!("iter.{}", name))),
    }
}

/// Возвращает либо (None, вектор значений) для кортежа,
/// либо (Some(числовые ключи), вектор значений) для struct-массива.
enum Coll {
    Tuple(Vec<Value>),
    Array(Vec<usize>, Vec<Value>),
}

fn collect(coll: &Value, fname: &str) -> IResult<Coll> {
    match coll {
        Value::Tuple(t) => Ok(Coll::Tuple(t.borrow().clone())),
        Value::Struct(s) => {
            let b = s.borrow();
            let mut keys: Vec<usize> = Vec::with_capacity(b.len());
            for k in b.keys() {
                let n: usize = k
                    .parse()
                    .map_err(|_| msg::bi_iter_not_array_key(k))?;
                keys.push(n);
            }
            keys.sort();
            let mut vals = Vec::with_capacity(keys.len());
            for k in &keys {
                vals.push(b[&k.to_string()].clone());
            }
            Ok(Coll::Array(keys, vals))
        }
        other => Err(msg::bi_bad_type(fname, "кортеж или struct-массив", other.type_name())),
    }
}

fn as_function(v: &Value, fname: &str) -> IResult<Value> {
    match v {
        Value::Function(_) | Value::Builtin(_) => Ok(v.clone()),
        other => Err(msg::bi_expected_function(fname, other.type_name())),
    }
}

fn call_map(interp: &mut Interpreter, args: Vec<Value>) -> IResult<Value> {
    let fname = "iter.map";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let coll = collect(&args[0], fname)?;
    let f = as_function(&args[1], fname)?;
    match coll {
        Coll::Tuple(vals) => {
            let mut out = Vec::with_capacity(vals.len());
            for v in vals {
                out.push(interp.call(f.clone(), vec![v])?);
            }
            Ok(Value::Tuple(Rc::new(RefCell::new(out))))
        }
        Coll::Array(keys, vals) => {
            let mut map = HashMap::new();
            for (k, v) in keys.iter().zip(vals.into_iter()) {
                let nv = interp.call(f.clone(), vec![v])?;
                map.insert(k.to_string(), nv);
            }
            Ok(Value::Struct(Rc::new(RefCell::new(map))))
        }
    }
}

fn call_filter(interp: &mut Interpreter, args: Vec<Value>) -> IResult<Value> {
    let fname = "iter.filter";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let coll = collect(&args[0], fname)?;
    let f = as_function(&args[1], fname)?;
    match coll {
        Coll::Tuple(vals) => {
            let mut out = Vec::new();
            for v in vals {
                if interp.call(f.clone(), vec![v.clone()])?.truthy() {
                    out.push(v);
                }
            }
            Ok(Value::Tuple(Rc::new(RefCell::new(out))))
        }
        Coll::Array(_keys, vals) => {
            // после filter индексы всегда 0..n-1
            let mut map = HashMap::new();
            let mut idx = 0usize;
            for v in vals {
                if interp.call(f.clone(), vec![v.clone()])?.truthy() {
                    map.insert(idx.to_string(), v);
                    idx += 1;
                }
            }
            Ok(Value::Struct(Rc::new(RefCell::new(map))))
        }
    }
}

fn call_reduce(interp: &mut Interpreter, args: Vec<Value>) -> IResult<Value> {
    let fname = "iter.reduce";
    if args.len() != 3 {
        return Err(msg::bi_arity(fname, 3, args.len()));
    }
    let coll = collect(&args[0], fname)?;
    let f = as_function(&args[1], fname)?;
    let init = args[2].clone();
    let vals: Vec<Value> = match coll {
        Coll::Tuple(v) => v,
        Coll::Array(_, v) => v,
    };
    let mut acc = init;
    for v in vals {
        acc = interp.call(f.clone(), vec![acc, v])?;
    }
    Ok(acc)
}