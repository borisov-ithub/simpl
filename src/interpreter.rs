use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::builtins;
use crate::env::{Env, EnvRef};
use crate::messages as msg;
use crate::value::{Function, Value};

pub struct Interpreter {
    pub globals: EnvRef,
    pub out: Rc<RefCell<Vec<u8>>>,
}

#[derive(Debug)]
pub(crate) enum Flow {
    Normal(Value),
    Break,
    Continue,
    Return(Value),
}

pub(crate) type IResult<T> = Result<T, String>;

impl Interpreter {
    pub fn new() -> Self {
        Self {
            globals: Env::new(),
            out: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn take_output(&self) -> String {
        String::from_utf8(self.out.borrow().clone()).unwrap_or_default()
    }

    pub fn run(&mut self, program: &Block) -> IResult<Value> {
        let env = self.globals.clone();
        let mut last = Value::Nil;
        for e in program {
            match self.eval(e, &env)? {
                Flow::Normal(v) => last = v,
                Flow::Return(v) => return Ok(v),
                Flow::Break => return Err(msg::rt_break_outside_loop()),
                Flow::Continue => return Err(msg::rt_continue_outside_loop()),
            }
        }
        Ok(last)
    }

    pub(crate) fn eval_block(&mut self, block: &Block, env: &EnvRef) -> IResult<Flow> {
        let mut last = Value::Nil;
        for e in block {
            match self.eval(e, env)? {
                Flow::Normal(v) => last = v,
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal(last))
    }

    fn eval(&mut self, expr: &Expr, env: &EnvRef) -> IResult<Flow> {
        match expr {
            Expr::Number(n) => Ok(Flow::Normal(Value::Number(*n))),
            Expr::Str(s) => Ok(Flow::Normal(Value::Str(s.clone()))),
            Expr::Bool(b) => Ok(Flow::Normal(Value::Bool(*b))),
            Expr::Nil => Ok(Flow::Normal(Value::Nil)),

            Expr::Interpolated(parts) => {
                let mut s = String::new();
                for p in parts {
                    match p {
                        StrPart::Lit(l) => s.push_str(l),
                        StrPart::Expr(e) => {
                            let v = self.eval_value(e, env)?;
                            s.push_str(&v.display());
                        }
                    }
                }
                Ok(Flow::Normal(Value::Str(s)))
            }

            Expr::Var(name) => match env.borrow().get(name) {
                Some(v) => Ok(Flow::Normal(v)),
                None => Err(msg::env_var_undefined(name)),
            },

            Expr::Tuple(elems) => {
                let mut out = Vec::new();
                for e in elems {
                    out.push(self.eval_value(e, env)?);
                }
                Ok(Flow::Normal(Value::Tuple(Rc::new(RefCell::new(out)))))
            }

            Expr::Struct(fields) => {
                let mut map = HashMap::new();
                let mut auto = 0usize;
                for f in fields {
                    let v = self.eval_value(&f.value, env)?;
                    let key = match &f.key {
                        Some(k) => k.clone(),
                        None => {
                            let k = auto.to_string();
                            auto += 1;
                            k
                        }
                    };
                    map.insert(key, v);
                }
                Ok(Flow::Normal(Value::Struct(Rc::new(RefCell::new(map)))))
            }

            Expr::Let { name, mutable, value } => {
                let v = self.eval_value(value, env)?;
                env.borrow_mut().define(name, v, *mutable);
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::LetTuple { names, value } => {
                let v = self.eval_value(value, env)?;
                let items: Vec<Value> = match &v {
                    Value::Tuple(t) => t.borrow().clone(),
                    other => return Err(msg::rt_forin_bad_iter(other.type_name())),
                };
                if items.len() != names.len() {
                    return Err(msg::rt_too_many_args(names.len(), items.len()));
                }
                for ((n, m), item) in names.iter().zip(items.into_iter()) {
                    env.borrow_mut().define(n, item, *m);
                }
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::Assign { name, value } => {
                let v = self.eval_value(value, env)?;
                env.borrow_mut().assign(name, v)?;
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::Unary { op, operand } => {
                let v = self.eval_value(operand, env)?;
                match op {
                    UnaryOp::Neg => match v {
                        Value::Number(n) => Ok(Flow::Normal(Value::Number(-n))),
                        other => Err(msg::rt_unary_neg_type(other.type_name())),
                    },
                    UnaryOp::Not => Ok(Flow::Normal(Value::Bool(!v.truthy()))),
                }
            }

            Expr::Binary { op, left, right } => {
                if let BinOp::And = op {
                    let l = self.eval_value(left, env)?;
                    if !l.truthy() {
                        return Ok(Flow::Normal(Value::Bool(false)));
                    }
                    let r = self.eval_value(right, env)?;
                    return Ok(Flow::Normal(Value::Bool(r.truthy())));
                }
                if let BinOp::Or = op {
                    let l = self.eval_value(left, env)?;
                    if l.truthy() {
                        return Ok(Flow::Normal(Value::Bool(true)));
                    }
                    let r = self.eval_value(right, env)?;
                    return Ok(Flow::Normal(Value::Bool(r.truthy())));
                }
                let l = self.eval_value(left, env)?;
                let r = self.eval_value(right, env)?;
                let v = self.binop(*op, l, r)?;
                Ok(Flow::Normal(v))
            }

            Expr::Index { target, index } => {
                let t = self.eval_value(target, env)?;
                let i = self.eval_value(index, env)?;
                self.index(t, i).map(Flow::Normal)
            }

            Expr::Field { target, name } => {
                let t = self.eval_value(target, env)?;
                self.field(t, name).map(Flow::Normal)
            }

            Expr::If { branches, else_branch } => {
                for (cond, body) in branches {
                    let c = self.eval_value(cond, env)?;
                    if c.truthy() {
                        return self.eval_block(body, &Env::child(env));
                    }
                }
                if let Some(b) = else_branch {
                    return self.eval_block(b, &Env::child(env));
                }
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::ForNum { var, from, to, step, body } => {
                let from_v = self.eval_value(from, env)?;
                let to_v = self.eval_value(to, env)?;
                let step_v = match step {
                    Some(s) => Some(self.eval_value(s, env)?),
                    None => None,
                };
                let f = as_num(&from_v, msg::rt_for_bad_type)?;
                let t = as_num(&to_v, msg::rt_for_bad_type)?;
                let st = match step_v {
                    Some(v) => as_num(&v, msg::rt_for_bad_type)?,
                    None => 1.0,
                };
                if st == 0.0 {
                    return Err(msg::rt_for_step_zero());
                }
                let mut i = f;
                let cond = |i: f64| if st > 0.0 { i <= t } else { i >= t };
                while cond(i) {
                    let child = Env::child(env);
                    child.borrow_mut().define(var, Value::Number(i), false);
                    match self.eval_block(body, &child)? {
                        Flow::Normal(_) => {}
                        Flow::Break => break,
                        Flow::Continue => {}
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                    }
                    i += st;
                }
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::ForIn { vars, iter, body } => {
                let it = self.eval_value(iter, env)?;
                let items: Vec<(Value, Value)> = match &it {
                    Value::Tuple(t) => {
                        let b = t.borrow();
                        b.iter()
                            .enumerate()
                            .map(|(i, v)| (Value::Number(i as f64), v.clone()))
                            .collect()
                    }
                    Value::Struct(s) => {
                        let b = s.borrow();
                        let mut v: Vec<(Value, Value)> = b
                            .iter()
                            .map(|(k, val)| (Value::Str(k.clone()), val.clone()))
                            .collect();
                        v.sort_by(|a, b| a.0.display().cmp(&b.0.display()));
                        v
                    }
                    other => return Err(msg::rt_forin_bad_iter(other.type_name())),
                };
                for (k, v) in items {
                    let child = Env::child(env);
                    if vars.len() == 1 {
                        child.borrow_mut().define(&vars[0], v, false);
                    } else {
                        child.borrow_mut().define(&vars[0], k, false);
                        child.borrow_mut().define(&vars[1], v, false);
                    }
                    match self.eval_block(body, &child)? {
                        Flow::Normal(_) => {}
                        Flow::Break => break,
                        Flow::Continue => {}
                        Flow::Return(val) => return Ok(Flow::Return(val)),
                    }
                }
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::Loop { body } => {
                loop {
                    match self.eval_block(body, &Env::child(env))? {
                        Flow::Normal(_) => {}
                        Flow::Break => break,
                        Flow::Continue => {}
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                    }
                }
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::Break => Ok(Flow::Break),
            Expr::Continue => Ok(Flow::Continue),

            Expr::Function { params, body } => {
                let f = Function {
                    params: params.clone(),
                    body: body.clone(),
                    env: env.clone(),
                };
                Ok(Flow::Normal(Value::Function(Rc::new(f))))
            }

            Expr::Lambda { params, body } => {
                let f = Function {
                    params: params.clone(),
                    body: body.clone(),
                    env: env.clone(),
                };
                Ok(Flow::Normal(Value::Function(Rc::new(f))))
            }

            Expr::Call { callee, args } => {
                let f = self.eval_value(callee, env)?;
                let mut vals = Vec::new();
                for a in args {
                    vals.push(self.eval_value(a, env)?);
                }
                let v = self.call(f, vals)?;
                Ok(Flow::Normal(v))
            }

            Expr::Match { subject, arms } => {
                let s = self.eval_value(subject, env)?;
                for arm in arms {
                    let mut bindings = Vec::new();
                    if pattern_match(&arm.pattern, &s, &mut bindings) {
                        let child = Env::child(env);
                        for (k, v) in bindings {
                            child.borrow_mut().define(&k, v, false);
                        }
                        if let Some(g) = &arm.guard {
                            let gv = self.eval_value(g, &child)?;
                            if !gv.truthy() {
                                continue;
                            }
                        }
                        return self.eval_block(&arm.body, &child);
                    }
                }
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::Echo { args, newline } => {
                let mut parts = Vec::new();
                for a in args {
                    let v = self.eval_value(a, env)?;
                    parts.push(v.display());
                }
                let mut s = parts.join(" ");
                if *newline {
                    s.push('\n');
                }
                self.out.borrow_mut().extend_from_slice(s.as_bytes());
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::Return(e) => {
                let v = match e {
                    Some(e) => self.eval_value(e, env)?,
                    None => Value::Nil,
                };
                Ok(Flow::Return(v))
            }

            Expr::Use { module, alias, names } => {
                self.eval_use(module, alias, names)
                    .map(|_| Flow::Normal(Value::Nil))
            }
        }
    }

    fn eval_use(
        &mut self,
        module: &str,
        alias: &Option<String>,
        names: &UseNames,
    ) -> Result<(), String> {
        let m = self
            .globals
            .borrow()
            .get(module)
            .ok_or_else(|| msg::use_unknown_module(module))?;
        let fields = match &m {
            Value::Struct(s) => s.borrow().clone(),
            _ => return Err(msg::use_unknown_module(module)),
        };

        let mut g = self.globals.borrow_mut();

        match names {
            UseNames::AliasOnly => {
                let a = alias.as_ref().unwrap();
                if g.get(a).is_some() {
                    return Err(msg::use_name_conflict(a));
                }
                g.define(a, m.clone(), false);
            }
            UseNames::Wildcard => {
                for (k, v) in &fields {
                    if g.get(k).is_some() {
                        return Err(msg::use_name_conflict(k));
                    }
                    g.define(k, v.clone(), false);
                }
                if let Some(a) = alias {
                    if g.get(a).is_some() {
                        return Err(msg::use_name_conflict(a));
                    }
                    g.define(a, m.clone(), false);
                }
            }
            UseNames::Explicit(list) => {
                for n in list {
                    let v = fields
                        .get(n)
                        .ok_or_else(|| msg::use_name_not_in_module(n, module))?;
                    if g.get(n).is_some() {
                        return Err(msg::use_name_conflict(n));
                    }
                    g.define(n, v.clone(), false);
                }
                if let Some(a) = alias {
                    if g.get(a).is_some() {
                        return Err(msg::use_name_conflict(a));
                    }
                    g.define(a, m.clone(), false);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn eval_value(&mut self, expr: &Expr, env: &EnvRef) -> IResult<Value> {
        match self.eval(expr, env)? {
            Flow::Normal(v) => Ok(v),
            Flow::Return(_) => Err(msg::rt_return_in_expr()),
            Flow::Break => Err(msg::rt_break_in_expr()),
            Flow::Continue => Err(msg::rt_continue_in_expr()),
        }
    }

    fn binop(&self, op: BinOp, l: Value, r: Value) -> IResult<Value> {
        use BinOp::*;
        match op {
            Add | Sub | Mul | Div | Mod => {
                let a = as_num(&l, msg::rt_arith_bad_type)?;
                let b = as_num(&r, msg::rt_arith_bad_type)?;
                Ok(Value::Number(match op {
                    Add => a + b,
                    Sub => a - b,
                    Mul => a * b,
                    Div => {
                        if b == 0.0 {
                            return Err(msg::rt_div_by_zero());
                        }
                        a / b
                    }
                    Mod => {
                        if b == 0.0 {
                            return Err(msg::rt_mod_by_zero());
                        }
                        a % b
                    }
                    _ => unreachable!(),
                }))
            }
            Concat => Ok(Value::Str(format!("{}{}", l.display(), r.display()))),
            Eq => Ok(Value::Bool(values_equal(&l, &r))),
            NotEq => Ok(Value::Bool(!values_equal(&l, &r))),
            Lt | LtEq | Gt | GtEq => {
                let a = as_num(&l, msg::rt_compare_bad_type)?;
                let b = as_num(&r, msg::rt_compare_bad_type)?;
                Ok(Value::Bool(match op {
                    Lt => a < b,
                    LtEq => a <= b,
                    Gt => a > b,
                    GtEq => a >= b,
                    _ => unreachable!(),
                }))
            }
            And | Or => unreachable!(),
        }
    }

    fn index(&self, target: Value, index: Value) -> IResult<Value> {
        match target {
            Value::Tuple(t) => {
                let i = as_num(&index, msg::rt_index_bad_type)? as usize;
                let b = t.borrow();
                b.get(i)
                    .cloned()
                    .ok_or_else(|| msg::rt_index_out_of_bounds(i, b.len()))
            }
            Value::Struct(s) => {
                let k = index.display();
                let b = s.borrow();
                b.get(&k)
                    .cloned()
                    .ok_or_else(|| msg::rt_field_missing(&k))
            }
            other => Err(msg::rt_index_on_non_collection(other.type_name())),
        }
    }

    fn field(&self, target: Value, name: &str) -> IResult<Value> {
        match target {
            Value::Struct(s) => {
                let b = s.borrow();
                b.get(name)
                    .cloned()
                    .ok_or_else(|| msg::rt_field_missing(name))
            }
            other => Err(msg::rt_field_on_non_struct(other.type_name())),
        }
    }

    pub(crate) fn call(&mut self, f: Value, args: Vec<Value>) -> IResult<Value> {
        match f {
            Value::Function(func) => {
                let call_env = Env::child(&func.env);
                let mut idx = 0;
                let required = func
                    .params
                    .iter()
                    .filter(|p| !p.star && p.default.is_none())
                    .count();

                for p in &func.params {
                    if p.star {
                        let rest: Vec<Value> = args[idx..].to_vec();
                        call_env.borrow_mut().define(
                            &p.name,
                            Value::Tuple(Rc::new(RefCell::new(rest))),
                            false,
                        );
                        idx = args.len();
                    } else {
                        let v = if idx < args.len() {
                            let v = args[idx].clone();
                            idx += 1;
                            v
                        } else if let Some(default) = &p.default {
                            self.eval_value(default, &call_env)?
                        } else {
                            return Err(msg::rt_too_few_args(required, args.len(), &p.name));
                        };
                        call_env.borrow_mut().define(&p.name, v, false);
                    }
                }

                if idx < args.len() {
                    return Err(msg::rt_too_many_args(idx, args.len()));
                }

                match self.eval_block(&func.body, &call_env)? {
                    Flow::Normal(v) => Ok(v),
                    Flow::Return(v) => Ok(v),
                    Flow::Break => Err(msg::rt_break_outside_loop()),
                    Flow::Continue => Err(msg::rt_continue_outside_loop()),
                }
            }
            Value::Builtin(name) => self.call_builtin(name, args),
            other => Err(msg::rt_call_non_function(other.type_name())),
        }
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> IResult<Value> {
        // базовые глобальные
        match name {
            "len" => return self.call_len(args),
            "type" => return self.call_type(args),
            "str" => return self.call_str(args),
            "num" => return self.call_num(args),
            "range" => return self.call_range(args),
            _ => {}
        }

        // модули по префиксу
        if let Some(rest) = name.strip_prefix("string.") {
            return builtins::string::call(self, rest, args);
        }
        if let Some(rest) = name.strip_prefix("math.") {
            return builtins::math::call(self, rest, args);
        }
        if let Some(rest) = name.strip_prefix("iter.") {
            return builtins::iter::call(self, rest, args);
        }
        if let Some(rest) = name.strip_prefix("struct.") {
            return builtins::struct_::call(self, rest, args);
        }
        if let Some(rest) = name.strip_prefix("tuple.") {
            return builtins::tuple_::call(self, rest, args);
        }
        if let Some(rest) = name.strip_prefix("file.") {
            return builtins::file::call(self, rest, args);
        }
        if let Some(rest) = name.strip_prefix("datetime.") {
            return builtins::datetime::call(self, rest, args);
        }

        Err(msg::bi_unknown(name))
    }

    fn call_len(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity("len", 1, args.len()));
        }
        match &args[0] {
            Value::Str(s) => Ok(Value::Number(s.chars().count() as f64)),
            Value::Tuple(t) => Ok(Value::Number(t.borrow().len() as f64)),
            Value::Struct(s) => Ok(Value::Number(s.borrow().len() as f64)),
            other => Err(msg::bi_bad_type("len", "строку, кортеж или struct", other.type_name())),
        }
    }

    fn call_type(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity("type", 1, args.len()));
        }
        Ok(Value::Str(args[0].type_name().to_string()))
    }

    fn call_str(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity("str", 1, args.len()));
        }
        Ok(Value::Str(args[0].display()))
    }

    fn call_num(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity("num", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(*n)),
            Value::Str(s) => s
                .parse::<f64>()
                .map(Value::Number)
                .map_err(|_| msg::bi_bad_type("num", "число", &format!("строку '{}'", s))),
            other => Err(msg::bi_bad_type("num", "число", other.type_name())),
        }
    }

    fn call_range(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 2 {
            return Err(msg::bi_arity("range", 2, args.len()));
        }
        let a = as_num(&args[0], msg::rt_arith_bad_type)? as i64;
        let b = as_num(&args[1], msg::rt_arith_bad_type)? as i64;
        let v: Vec<Value> = (a..b).map(|i| Value::Number(i as f64)).collect();
        Ok(Value::Tuple(Rc::new(RefCell::new(v))))
    }
}

fn as_num(v: &Value, msg_fn: fn(&str) -> String) -> IResult<f64> {
    match v {
        Value::Number(n) => Ok(*n),
        other => Err(msg_fn(other.type_name())),
    }
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

fn pattern_match(p: &Pattern, v: &Value, bindings: &mut Vec<(String, Value)>) -> bool {
    match (p, v) {
        (Pattern::Wildcard, _) => true,
        (Pattern::Var(name), _) => {
            bindings.push((name.clone(), v.clone()));
            true
        }
        (Pattern::Number(n), Value::Number(m)) => n == m,
        (Pattern::Str(s), Value::Str(m)) => s == m,
        (Pattern::Bool(b), Value::Bool(m)) => b == m,
        (Pattern::Nil, Value::Nil) => true,
        (Pattern::Tuple(ps), Value::Tuple(vs)) => {
            let vb = vs.borrow();
            if ps.len() != vb.len() {
                return false;
            }
            for (p, v) in ps.iter().zip(vb.iter()) {
                if !pattern_match(p, v, bindings) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}