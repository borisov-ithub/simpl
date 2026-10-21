use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Timelike};

use crate::interpreter::{Interpreter, IResult};
use crate::messages as msg;
use crate::value::Value;

pub fn call(_interp: &mut Interpreter, name: &str, args: Vec<Value>) -> IResult<Value> {
    match name {
        "now" => call_now(),
        "format" => call_format(args),
        "timestamp" => call_timestamp(args),
        "from_timestamp" => call_from_timestamp(args),
        _ => Err(msg::bi_unknown(&format!("datetime.{}", name))),
    }
}

fn num_field(v: f64) -> Value {
    Value::Number(v)
}

fn dt_to_struct(dt: &NaiveDateTime) -> Value {
    let mut map = HashMap::new();
    map.insert("year".to_string(), num_field(dt.year() as f64));
    map.insert("month".to_string(), num_field(dt.month() as f64));
    map.insert("day".to_string(), num_field(dt.day() as f64));
    map.insert("hour".to_string(), num_field(dt.hour() as f64));
    map.insert("min".to_string(), num_field(dt.minute() as f64));
    map.insert("sec".to_string(), num_field(dt.second() as f64));
    map.insert("wday".to_string(), num_field(dt.weekday().num_days_from_monday() as f64 + 1.0));
    Value::Struct(Rc::new(RefCell::new(map)))
}

fn struct_to_dt(v: &Value) -> IResult<NaiveDateTime> {
    let s = match v {
        Value::Struct(s) => s.borrow().clone(),
        other => return Err(msg::bi_bad_type("datetime", "struct", other.type_name())),
    };
    let get = |name: &str| -> IResult<i32> {
        match s.get(name) {
            Some(Value::Number(n)) => Ok(*n as i32),
            _ => Err(msg::bi_datetime_bad_struct(name)),
        }
    };
    let y = get("year")?;
    let m = get("month")? as u32;
    let d = get("day")? as u32;
    let h = if s.contains_key("hour") { get("hour")? as u32 } else { 0 };
    let mi = if s.contains_key("min") { get("min")? as u32 } else { 0 };
    let se = if s.contains_key("sec") { get("sec")? as u32 } else { 0 };
    chrono::NaiveDate::from_ymd_opt(y, m, d)
        .and_then(|nd| nd.and_hms_opt(h, mi, se))
        .ok_or_else(msg::bi_datetime_bad_timestamp)
}

fn call_now() -> IResult<Value> {
    let now = Local::now().naive_local();
    Ok(dt_to_struct(&now))
}

fn call_format(args: Vec<Value>) -> IResult<Value> {
    let fname = "datetime.format";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let dt = struct_to_dt(&args[0])?;
    let fmt = match &args[1] {
        Value::Str(s) => s.clone(),
        other => return Err(msg::bi_bad_type(fname, "строку формата", other.type_name())),
    };
    // chrono использует %Y, %m, %d, %H, %M, %S — совпадает с нашим форматом
    Ok(Value::Str(dt.format(&fmt).to_string()))
}

fn call_timestamp(args: Vec<Value>) -> IResult<Value> {
    let fname = "datetime.timestamp";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let dt = struct_to_dt(&args[0])?;
    let ts = Local
        .from_local_datetime(&dt)
        .single()
        .ok_or_else(msg::bi_datetime_bad_timestamp)?
        .timestamp();
    Ok(Value::Number(ts as f64))
}

fn call_from_timestamp(args: Vec<Value>) -> IResult<Value> {
    let fname = "datetime.from_timestamp";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let ts = match &args[0] {
        Value::Number(n) => *n as i64,
        other => return Err(msg::bi_bad_type(fname, "число", other.type_name())),
    };
    let dt = Local
        .timestamp_opt(ts, 0)
        .single()
        .ok_or_else(msg::bi_datetime_bad_timestamp)?
        .naive_local();
    Ok(dt_to_struct(&dt))
}