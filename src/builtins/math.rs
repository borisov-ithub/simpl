use rand::Rng;

use crate::interpreter::{Interpreter, IResult};
use crate::messages as msg;
use crate::value::Value;

pub fn call(_interp: &mut Interpreter, name: &str, args: Vec<Value>) -> IResult<Value> {
    match name {
        "abs" => num_map1(args, "math.abs", |x| x.abs()),
        "floor" => num_map1(args, "math.floor", |x| x.floor()),
        "ceil" => num_map1(args, "math.ceil", |x| x.ceil()),
        "round" => call_round(args),
        "sqrt" => call_sqrt(args),
        "log" => call_log(args),
        "min" => call_minmax(args, true),
        "max" => call_minmax(args, false),
        "random" => call_random(args),
        "randomseed" => call_randomseed(args),
        _ => Err(msg::bi_unknown(&format!("math.{}", name))),
    }
}

fn as_num(v: &Value, fname: &str) -> IResult<f64> {
    match v {
        Value::Number(n) => Ok(*n),
        other => Err(msg::bi_bad_type(fname, "число", other.type_name())),
    }
}

fn as_int(v: &Value, fname: &str) -> IResult<i64> {
    match v {
        Value::Number(n) if n.fract() == 0.0 && n.is_finite() => Ok(*n as i64),
        other => Err(msg::bi_bad_type(fname, "целое число", other.type_name())),
    }
}

fn num_map1<F: Fn(f64) -> f64>(args: Vec<Value>, fname: &str, f: F) -> IResult<Value> {
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let x = as_num(&args[0], fname)?;
    Ok(Value::Number(f(x)))
}

fn call_round(args: Vec<Value>) -> IResult<Value> {
    let fname = "math.round";
    if args.is_empty() || args.len() > 2 {
        return Err(msg::bi_arity_range(fname, 1, 2, args.len()));
    }
    let x = as_num(&args[0], fname)?;
    let digits = if args.len() == 2 { as_int(&args[1], fname)? } else { 0 };
    if digits < 0 {
        return Err(msg::bi_math_round_negative_digits(digits));
    }
    let factor = 10f64.powi(digits as i32);
    let rounded = (x * factor).round() / factor;
    Ok(Value::Number(rounded))
}

fn call_sqrt(args: Vec<Value>) -> IResult<Value> {
    let fname = "math.sqrt";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let x = as_num(&args[0], fname)?;
    if x < 0.0 {
        return Err(msg::bi_math_sqrt_negative());
    }
    Ok(Value::Number(x.sqrt()))
}

fn call_log(args: Vec<Value>) -> IResult<Value> {
    let fname = "math.log";
    if args.len() != 2 {
        return Err(msg::bi_arity(fname, 2, args.len()));
    }
    let x = as_num(&args[0], fname)?;
    let base = as_num(&args[1], fname)?;
    if x <= 0.0 {
        return Err(msg::bi_math_log_bad_arg(x));
    }
    if base <= 0.0 || base == 1.0 {
        return Err(msg::bi_math_log_bad_base(base));
    }
    Ok(Value::Number(x.log(base)))
}

fn call_minmax(args: Vec<Value>, is_min: bool) -> IResult<Value> {
    let fname = if is_min { "math.min" } else { "math.max" };
    if args.is_empty() {
        return Err(msg::bi_math_empty_args());
    }
    let mut acc = as_num(&args[0], fname)?;
    for v in &args[1..] {
        let x = as_num(v, fname)?;
        acc = if is_min { acc.min(x) } else { acc.max(x) };
    }
    Ok(Value::Number(acc))
}

fn call_random(args: Vec<Value>) -> IResult<Value> {
    let fname = "math.random";
    let mut rng = rand::thread_rng();
    match args.len() {
        0 => Ok(Value::Number(rng.gen::<f64>())),
        1 => {
            let n = as_num(&args[0], fname)?;
            if n <= 0.0 {
                return Err(msg::bi_bad_type(fname, "положительное число", &format!("{}", n)));
            }
            let k = rng.gen_range(0..(n as i64));
            Ok(Value::Number(k as f64))
        }
        2 => {
            let a = as_num(&args[0], fname)? as i64;
            let b = as_num(&args[1], fname)? as i64;
            if a >= b {
                return Err(msg::bi_bad_type(fname, "диапазон a < b", &format!("{}..{}", a, b)));
            }
            Ok(Value::Number(rng.gen_range(a..b) as f64))
        }
        n => Err(msg::bi_arity_range(fname, 0, 2, n)),
    }
}

fn call_randomseed(args: Vec<Value>) -> IResult<Value> {
    let fname = "math.randomseed";
    if args.len() != 1 {
        return Err(msg::bi_arity(fname, 1, args.len()));
    }
    let _ = as_num(&args[0], fname)?;
    Ok(Value::Nil)
}