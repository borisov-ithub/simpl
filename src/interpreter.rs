use std::cell::RefCell;
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use std::path::{PathBuf};

use crate::ast::*;
use crate::builtins;
use crate::env::{Env, EnvRef};
use crate::messages as msg;
use crate::value::{Function, OrderedKey, StructMap, Value};

pub struct Interpreter {
    pub globals: EnvRef,
    pub out: Rc<RefCell<Vec<u8>>>,
    pub echo_to_stdout: bool,
    pub current_file: Option<PathBuf>,
    pub module_cache: Rc<RefCell<HashMap<PathBuf, Value>>>,
    pub loading: Rc<RefCell<HashSet<PathBuf>>>,
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
            echo_to_stdout: false,
            current_file: None,
            module_cache: Rc::new(RefCell::new(HashMap::new())),
            loading: Rc::new(RefCell::new(HashSet::new())),
        }
    }

    pub fn take_output(&self) -> String {
        String::from_utf8(self.out.borrow().clone()).unwrap_or_default()
    }

    pub fn run(&mut self, program: &Block) -> IResult<Value> {
        let env = self.globals.clone();
        self.run_in_env(program, &env)
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
            
            Expr::Pipe { left, right } => {
                let v = self.eval_value(left, env)?;
                let f = self.eval_value(right, env)?;
                let result = self.call(f, vec![v])?;
                Ok(Flow::Normal(result))
            }
            
            Expr::Compose { left, right } => {
                let f = self.eval_value(left, env)?;
                let g = self.eval_value(right, env)?;
                // проверяем, что оба — функции
                if !matches!(f, Value::Function(_) | Value::Builtin(_) | Value::Flipped { .. } | Value::Composed { .. }) {
                    return Err(msg::rt_compose_non_function(f.type_name()));
                }
                if !matches!(g, Value::Function(_) | Value::Builtin(_) | Value::Flipped { .. } | Value::Composed { .. }) {
                    return Err(msg::rt_compose_non_function(g.type_name()));
                }
                Ok(Flow::Normal(Value::Composed {
                    left: Rc::new(f),
                    right: Rc::new(g),
                }))
            }

            Expr::TypeOf(operand) => {
                let v = self.eval_value(operand, env)?;
                Ok(Flow::Normal(Value::Str(v.type_name().to_string())))
            }
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
                let mut map = StructMap::new();
                let mut max_num: Option<i64> = None;
                for f in fields {
                    let v = self.eval_value(&f.value, env)?;
                    let key = match &f.key {
                        Some(k) => OrderedKey::from_str(k),
                        None => {
                            let k = match max_num {
                                Some(m) => m + 1,
                                None => 0,
                            };
                            OrderedKey(k.to_string())
                        }
                    };
                    if let Ok(n) = key.0.parse::<i64>() {
                        max_num = Some(max_num.map_or(n, |m| m.max(n)));
                    }
                    map.insert(key, v);  // перезапись при конфликте — ок
                }
                Ok(Flow::Normal(Value::Struct(Rc::new(RefCell::new(map)))))
            }

            Expr::StructComprehension { key, value, vars, iter, condition } => {
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
                        b.iter()
                            .map(|(k, v)| (key_to_value(k), v.clone()))
                            .collect()
                    }
                    other => return Err(msg::rt_forin_bad_iter(other.type_name())),
                };
                let mut map = StructMap::new();
                for (k, v) in items {
                    let child = Env::child(env);
                    if vars.len() == 1 {
                        child.borrow_mut().define_new(&vars[0], v, false)?;
                    } else {
                        child.borrow_mut().define_new(&vars[0], k, false)?;
                        child.borrow_mut().define_new(&vars[1], v, false)?;
                    }
                    if let Some(cond) = condition {
                        let c = self.eval_value(cond, &child)?;
                        if !c.truthy() {
                            continue;
                        }
                    }
                    let kk = self.eval_value(key, &child)?;
                    let vv = self.eval_value(value, &child)?;
                    map.insert(OrderedKey(to_key_string(&kk)), vv);
                }
                Ok(Flow::Normal(Value::Struct(Rc::new(RefCell::new(map)))))
            }

            Expr::Let { name, mutable, value } => {
                let v = self.eval_value(value, env)?;
                env.borrow_mut().define_new(name, v, *mutable)?;
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
                    env.borrow_mut().define_new(n, item, *m)?;
                }
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::Assign { name, value } => {
                let v = self.eval_value(value, env)?;
                env.borrow_mut().assign(name, v)?;
                Ok(Flow::Normal(Value::Nil))
            }

            Expr::AssignField { target, key, value } => {
                let v = self.eval_value(value, env)?;
                let t = self.eval_value(target, env)?;
                match (t, key) {
                    (Value::Struct(s), AssignKey::Field(name)) => {
                        s.borrow_mut().insert(OrderedKey::from_str(name), v);
                        Ok(Flow::Normal(Value::Nil))
                    }
                    (Value::Struct(s), AssignKey::Index(idx_expr)) => {
                        let idx = self.eval_value(idx_expr, env)?;
                        let k = OrderedKey(to_key_string(&idx));
                        s.borrow_mut().insert(k, v);
                        Ok(Flow::Normal(Value::Nil))
                    }
                    (Value::Tuple(_), _) => Err(msg::rt_assign_field_not_struct("tuple")),
                    (other, _) => Err(msg::rt_assign_field_not_struct(other.type_name())),
                }
            }

            Expr::MethodDef { target, name, params, body } => {
                let t = self.eval_value(target, env)?;
                let func = Function {
                    params: params.clone(),
                    body: body.clone(),
                    env: env.clone(),
                    bound: Vec::new(),
                };
                let v = Value::Function(Rc::new(func));
                match t {
                    Value::Struct(s) => {
                        s.borrow_mut().insert(OrderedKey::from_str(name), v);
                        Ok(Flow::Normal(Value::Nil))
                    }
                    other => Err(msg::rt_assign_field_not_struct(other.type_name())),
                }
            }

            Expr::Unary { op, operand } => {
                let v = self.eval_value(operand, env)?;
                match op {
                    UnaryOp::Neg => match v {
                        Value::Number(n) => Ok(Flow::Normal(Value::Number(-n))),
                        other => Err(msg::rt_unary_neg_type(other.type_name())),
                    },
                    UnaryOp::Not => Ok(Flow::Normal(Value::Bool(!v.truthy()))),
                    UnaryOp::BitNot => {
                        let n = as_i64(&v, "~")?;
                        Ok(Flow::Normal(Value::Number((!n) as f64)))
                    }
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
                    child.borrow_mut().define_new(var, Value::Number(i), false)?;
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
                        b.iter()
                            .map(|(k, v)| (key_to_value(k), v.clone()))
                            .collect()
                    }
                    other => return Err(msg::rt_forin_bad_iter(other.type_name())),
                };
                for (k, v) in items {
                    let child = Env::child(env);
                    if vars.len() == 1 {
                        child.borrow_mut().define_new(&vars[0], v, false)?;
                    } else {
                        child.borrow_mut().define_new(&vars[0], k, false)?;
                        child.borrow_mut().define_new(&vars[1], v, false)?;
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
                    bound: Vec::new(),
                };
                Ok(Flow::Normal(Value::Function(Rc::new(f))))
            }

            Expr::Lambda { params, body } => {
                let f = Function {
                    params: params.clone(),
                    body: body.clone(),
                    env: env.clone(),
                    bound: Vec::new(),
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
                            child.borrow_mut().define_new(&k, v, false)?;
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
                if self.echo_to_stdout {
                    print!("{}", s);
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                } else {
                    self.out.borrow_mut().extend_from_slice(s.as_bytes());
                }
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

            Expr::Import { path, alias, names } => {
                let module = self.load_module(path)?;
                let module_name = format!("import {}", path);
                self.bind_names(module, alias, names, &module_name)
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
        self.bind_names(m, alias, names, module)
    }
    
    fn bind_names(
        &mut self,
        m: Value,
        alias: &Option<String>,
        names: &UseNames,
        module_name: &str,
    ) -> Result<(), String> {
        let fields = match &m {
            Value::Struct(s) => s.borrow().clone(),
            _ => return Err(msg::use_unknown_module(module_name)),
        };
    
        let mut g = self.globals.borrow_mut();
    
        match names {
            UseNames::AliasOnly => {
                let a = alias.as_ref().unwrap();
                g.define_new(a, m.clone(), false)?;
            }
            UseNames::Wildcard => {
                for (k, v) in &fields {
                    g.define_new(&k.0, v.clone(), false)?;
                }
                if let Some(a) = alias {
                    g.define_new(a, m.clone(), false)?;
                }
            }
            UseNames::Explicit(list) => {
                for n in list {
                    let key = OrderedKey::from_str(n);
                    let v = fields
                        .get(&key)
                        .ok_or_else(|| msg::use_name_not_in_module(n, module_name))?;
                    g.define_new(n, v.clone(), false)?;
                }
                if let Some(a) = alias {
                    g.define_new(a, m.clone(), false)?;
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
            BitAnd | BitOr | BitXor => {
                let a = as_i64(&l, "побитовое И/ИЛИ/XOR")?;
                let b = as_i64(&r, "побитовое И/ИЛИ/XOR")?;
                let result = match op {
                    BitAnd => a & b,
                    BitOr => a | b,
                    BitXor => a ^ b,
                    _ => unreachable!(),
                };
                Ok(Value::Number(result as f64))
            }
            Shl | Shr => {
                let a = as_i64(&l, "сдвиг")?;
                let n = as_i64(&r, "сдвиг")?;
                if n < 0 || n >= 64 {
                    return Err(msg::rt_shift_too_big(
                        if let BinOp::Shl = op { "<<" } else { ">>" },
                        n,
                    ));
                }
                let result = match op {
                    Shl => a.wrapping_shl(n as u32),
                    Shr => a.wrapping_shr(n as u32),
                    _ => unreachable!(),
                };
                Ok(Value::Number(result as f64))
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
                let key = OrderedKey(to_key_string(&index));
                let b = s.borrow();
                b.get(&key)
                    .cloned()
                    .ok_or_else(|| msg::rt_field_missing(&key.0))
            }
            other => Err(msg::rt_index_on_non_collection(other.type_name())),
        }
    }

    fn field(&self, target: Value, name: &str) -> IResult<Value> {
        match target {
            Value::Struct(s) => {
                let key = OrderedKey::from_str(name);
                let b = s.borrow();
                b.get(&key)
                    .cloned()
                    .ok_or_else(|| msg::rt_field_missing(name))
            }
            other => Err(msg::rt_field_on_non_struct(other.type_name())),
        }
    }

    pub(crate) fn call(&mut self, f: Value, args: Vec<Value>) -> IResult<Value> {
        match f {
            Value::Function(func) => self.call_function(func, args),
            Value::Builtin(name) => self.call_builtin(name, args),
            Value::Flipped { inner, bound } => {
                let mut all = bound.clone();
                all.extend(args);
        
                let required = match &*inner {
                    Value::Function(func) => func
                        .params
                        .iter()
                        .filter(|p| !p.star && p.default.is_none())
                        .count(),
                    _ => return Err(msg::rt_flip_non_function(inner.type_name())),
                };
                if required != 2 {
                    return Err(msg::rt_flip_needs_binary(required));
                }
        
                if all.len() < 2 {
                    return Ok(Value::Flipped { inner: inner.clone(), bound: all });
                }
                all.swap(0, 1);
                self.call((*inner).clone(), all)
            }
            Value::Composed { left, right } => {
                let inner = self.call((*left).clone(), args)?;
                self.call((*right).clone(), vec![inner])
            }
            other => Err(msg::rt_call_non_function(other.type_name())),
        }
    }
    
    fn call_function(&mut self, func: Rc<Function>, args: Vec<Value>) -> IResult<Value> {
        let mut all_args: Vec<Value> = func.bound.clone();
        all_args.extend(args);
    
        // Считаем, сколько обязательных параметров (без *, без default)
        let required = func
            .params
            .iter()
            .filter(|p| !p.star && p.default.is_none())
            .count();
    
        // Есть ли *args
        // let has_star = func.params.iter().any(|p| p.star);
    
        // Каррирование: если не хватает обязательных аргументов
        if all_args.len() < required {
            // Возвращаем новую функцию с привязанными аргументами
            let new_func = Function {
                params: func.params.clone(),
                body: func.body.clone(),
                env: func.env.clone(),
                bound: all_args,
            };
            return Ok(Value::Function(Rc::new(new_func)));
        }
    
        // Полный вызов
        let call_env = Env::child(&func.env);
        let mut idx = 0;
    
        for p in &func.params {
            if p.star {
                let rest: Vec<Value> = all_args[idx..].to_vec();
                call_env.borrow_mut().define(
                    &p.name,
                    Value::Tuple(Rc::new(RefCell::new(rest))),
                    false,
                );
                idx = all_args.len();
            } else {
                let v = if idx < all_args.len() {
                    let v = all_args[idx].clone();
                    idx += 1;
                    v
                } else if let Some(default) = &p.default {
                    self.eval_value(default, &call_env)?
                } else {
                    // Теоретически недостижимо: required уже проверен
                    return Err(msg::rt_too_few_args(required, all_args.len(), &p.name));
                };
                call_env.borrow_mut().define(&p.name, v, false);
            }
        }
    
        if idx < all_args.len() {
            return Err(msg::rt_too_many_args(idx, all_args.len()));
        }
    
        match self.eval_block(&func.body, &call_env)? {
            Flow::Normal(v) => Ok(v),
            Flow::Return(v) => Ok(v),
            Flow::Break => Err(msg::rt_break_outside_loop()),
            Flow::Continue => Err(msg::rt_continue_outside_loop()),
        }
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> IResult<Value> {
        match name {
            "len" => return self.call_len(args),
            "str" => return self.call_str(args),
            "num" => return self.call_num(args),
            "range" => return self.call_range(args),
            "input" => return self.call_input(args),
            "bin" => return self.call_bin(args),
            "hex" => return self.call_hex(args),
            "oct" => return self.call_oct(args),
            "ord" => return self.call_ord(args),
            "chr" => return self.call_chr(args),
            "sleep" => return self.call_sleep(args),
            "flip" => return self.call_flip(args),
            "pcall" => return self.call_pcall(args),
            "xpcall" => return self.call_xpcall(args),
            "error" => return self.call_error(args),
            _ => {}
        }
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
            Value::Str(s) => parse_num(s)
                .map(Value::Number)
                .ok_or_else(|| msg::bi_bad_type("num", "число", &format!("строку '{}'", s))),
            other => Err(msg::bi_bad_type("num", "число", other.type_name())),
        }
    }

    fn call_bin(&mut self, args: Vec<Value>) -> IResult<Value> {
        self.format_radix(args, 2, "0b", "bin")
    }
    
    fn call_oct(&mut self, args: Vec<Value>) -> IResult<Value> {
        self.format_radix(args, 8, "0o", "oct")
    }
    
    fn call_hex(&mut self, args: Vec<Value>) -> IResult<Value> {
        self.format_radix(args, 16, "0x", "hex")
    }
    
    fn format_radix(&mut self, args: Vec<Value>, radix: u32, prefix: &str, fname: &str) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity(fname, 1, args.len()));
        }
        let n = match &args[0] {
            Value::Number(n) if n.fract() == 0.0 && n.is_finite() => *n as i64,
            other => return Err(msg::bi_bad_type(fname, "целое число", other.type_name())),
        };
        let sign = if n < 0 { "-" } else { "" };
        let abs = n.unsigned_abs();
        let digits = match radix {
            2 => format!("{:b}", abs),
            8 => format!("{:o}", abs),
            16 => format!("{:x}", abs),
            _ => unreachable!(),
        };
        Ok(Value::Str(format!("{}{}{}", sign, prefix, digits)))
    }
    
    fn call_ord(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity("ord", 1, args.len()));
        }
        let s = match &args[0] {
            Value::Str(s) => s,
            other => return Err(msg::bi_bad_type("ord", "строку", other.type_name())),
        };
        let mut chars = s.chars();
        let c = match (chars.next(), chars.next()) {
            (Some(c), None) => c,
            (None, _) => return Err(msg::bi_ord_empty()),
            _ => return Err(msg::bi_ord_too_long(s)),
        };
        Ok(Value::Number(c as u32 as f64))
    }
    
    fn call_chr(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity("chr", 1, args.len()));
        }
        let n = match &args[0] {
            Value::Number(n) if n.fract() == 0.0 && *n >= 0.0 && n.is_finite() => *n as u32,
            other => return Err(msg::bi_bad_type("chr", "неотрицательное целое число", other.type_name())),
        };
        match char::from_u32(n) {
            Some(c) => Ok(Value::Str(c.to_string())),
            None => Err(msg::bi_chr_invalid(n)),
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

    fn call_input(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() > 1 {
            return Err(msg::bi_arity_range("input", 0, 1, args.len()));
        }
        let prompt = if args.len() == 1 {
            match &args[0] {
                Value::Str(s) => s.clone(),
                other => return Err(msg::bi_bad_type("input", "строку-приглашение", other.type_name())),
            }
        } else {
            String::new()
        };
    
        if !prompt.is_empty() {
            print!("{}", msg::input_prompt(&prompt));
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .map_err(|e| msg::input_read_error(&e))?;
        // убираем завершающий \n и \r\n
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        Ok(Value::Str(line))
    }

    fn call_sleep(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity("sleep", 1, args.len()));
        }
        let secs = match &args[0] {
            Value::Number(n) if n.is_finite() && *n >= 0.0 => *n,
            Value::Number(n) => {
                return Err(msg::bi_bad_type(
                    "sleep",
                    "неотрицательное конечное число",
                    &format!("{}", n),
                ))
            }
            other => {
                return Err(msg::bi_bad_type("sleep", "число", other.type_name()))
            }
        };
        const MAX_SLEEP_SECS: f64 = 1e9;
        if secs > MAX_SLEEP_SECS {
            return Err(msg::bi_bad_type(
                "sleep",
                "разумное число секунд (<= 1e9)",
                &format!("{}", secs),
            ));
        }
        if secs > 0.0 {
            std::thread::sleep(std::time::Duration::from_secs_f64(secs));
        }
        Ok(Value::Nil)
    }

    fn call_flip(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity("flip", 1, args.len()));
        }
        match &args[0] {
            Value::Function(f) => {
                let required = f.params.iter().filter(|p| !p.star && p.default.is_none()).count();
                if required != 2 {
                    return Err(msg::rt_flip_needs_binary(required));
                }
                Ok(Value::Flipped {
                    inner: Rc::new(args[0].clone()),
                    bound: Vec::new(),
                })
            }
            Value::Builtin(_) => Err(msg::rt_flip_builtin()),
            Value::Flipped { .. } => Err(msg::rt_flip_non_function("flipped function")),
            other => Err(msg::rt_flip_non_function(other.type_name())),
        }
    }

    fn call_pcall(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.is_empty() {
            return Err(msg::bi_arity_range("pcall", 1, usize::MAX, 0));
        }
        let f = args[0].clone();
        let rest: Vec<Value> = args[1..].to_vec();
        match self.call(f, rest) {
            Ok(v) => {
                let pair = vec![Value::Bool(true), v];
                Ok(Value::Tuple(Rc::new(RefCell::new(pair))))
            }
            Err(e) => {
                let pair = vec![Value::Bool(false), Value::Str(e)];
                Ok(Value::Tuple(Rc::new(RefCell::new(pair))))
            }
        }
    }
    
    fn call_xpcall(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() < 2 {
            return Err(msg::bi_arity_range("xpcall", 2, usize::MAX, args.len()));
        }
        let f = args[0].clone();
        let handler = args[1].clone();
        let rest: Vec<Value> = args[2..].to_vec();
        match self.call(f, rest) {
            Ok(v) => {
                let pair = vec![Value::Bool(true), v];
                Ok(Value::Tuple(Rc::new(RefCell::new(pair))))
            }
            Err(e) => {
                // вызываем handler(e)
                match self.call(handler, vec![Value::Str(e.clone())]) {
                    Ok(hv) => {
                        let pair = vec![Value::Bool(false), hv];
                        Ok(Value::Tuple(Rc::new(RefCell::new(pair))))
                    }
                    Err(he) => {
                        // handler сам упал — возвращаем ошибку handler
                        let pair = vec![Value::Bool(false), Value::Str(he)];
                        Ok(Value::Tuple(Rc::new(RefCell::new(pair))))
                    }
                }
            }
        }
    }
    
    fn call_error(&mut self, args: Vec<Value>) -> IResult<Value> {
        if args.len() != 1 {
            return Err(msg::bi_arity("error", 1, args.len()));
        }
        let m = match &args[0] {
            Value::Str(s) => s.clone(),
            other => other.display(),
        };
        Err(msg::rt_user_error(&m))
    }

    fn resolve_import_path(&self, path: &str) -> IResult<PathBuf> {
        let p = PathBuf::from(path);
        let abs = if p.is_absolute() {
            p
        } else if let Some(base) = &self.current_file {
            base.parent()
                .map(|d| d.join(&p))
                .unwrap_or_else(|| p.clone())
        } else {
            p
        };
        let abs = if abs.extension().is_some() {
            abs
        } else {
            abs.with_extension("spl")
        };
        abs.canonicalize()
            .map_err(|e| msg::import_read_error(&abs.display().to_string(), &e))
    }
    
    fn load_module(&mut self, path: &str) -> IResult<Value> {
        let full_path = self.resolve_import_path(path)?;
    
        if let Some(cached) = self.module_cache.borrow().get(&full_path) {
            return Ok(cached.clone());
        }
        if self.loading.borrow().contains(&full_path) {
            return Err(msg::import_cycle(&full_path.display().to_string()));
        }
        self.loading.borrow_mut().insert(full_path.clone());
    
        let src = std::fs::read_to_string(&full_path)
            .map_err(|e| msg::import_read_error(&full_path.display().to_string(), &e))?;
    
        let mut lexer = crate::lexer::Lexer::new(&src);
        let tokens = lexer.tokenize()?;
        let mut parser = crate::parser::Parser::new(tokens);
        let program = parser.parse_program()?;
    
        let module_env = Env::new();
        {
            let mut g = module_env.borrow_mut();
            crate::builtins::register(&mut g.vars);
        }
        let initial: HashSet<String> = module_env.borrow().vars.keys().cloned().collect();
    
        let old_file = self.current_file.clone();
        self.current_file = Some(full_path.clone());
        let run_result = self.run_in_env(&program, &module_env);
        self.current_file = old_file;
        run_result?;
    
        let final_vars: HashMap<String, (Value, bool)> = module_env.borrow().vars.clone();
        let mut fields = StructMap::new();
        for (name, (val, _)) in final_vars {
            if initial.contains(&name) { continue; }
            fields.insert(OrderedKey(name), val);
        }
        let module = Value::Struct(Rc::new(RefCell::new(fields)));
    
        self.module_cache.borrow_mut().insert(full_path.clone(), module.clone());
        self.loading.borrow_mut().remove(&full_path);
        Ok(module)
    }
    
    pub fn run_in_env(&mut self, program: &Block, env: &EnvRef) -> IResult<Value> {
        let mut last = Value::Nil;
        for e in program {
            match self.eval(e, env)? {
                Flow::Normal(v) => last = v,
                Flow::Return(v) => return Ok(v),
                Flow::Break => return Err(msg::rt_break_outside_loop()),
                Flow::Continue => return Err(msg::rt_continue_outside_loop()),
            }
        }
        Ok(last)
    }
}

pub(crate) fn as_i64(v: &Value, ctx: &str) -> IResult<i64> {
    match v {
        Value::Number(n) => {
            if !n.is_finite() || n.fract() != 0.0 {
                return Err(msg::rt_bitop_expected_int(ctx, &format!("{}", n)));
            }
            // f64 хранит целые точно до 2^53. 
            // abs() >= 2^53 — потеря точности.
            if n.abs() >= 9_007_199_254_740_992.0 {
                return Err(msg::rt_bitop_overflow(ctx, *n));
            }
            Ok(*n as i64)
        }
        other => Err(msg::rt_bitop_expected_int(ctx, other.type_name())),
    }
}

pub(crate) fn as_num(v: &Value, msg_fn: fn(&str) -> String) -> IResult<f64> {
    match v {
        Value::Number(n) => Ok(*n),
        other => Err(msg_fn(other.type_name())),
    }
}

pub(crate) fn key_to_value(k: &OrderedKey) -> Value {
    match k.0.parse::<i64>() {
        Ok(n) => Value::Number(n as f64),
        Err(_) => Value::Str(k.0.clone()),
    }
}

pub(crate) fn to_key_string(v: &Value) -> String {
    match v {
        Value::Str(s) => s.clone(),
        Value::Number(n) if n.fract() == 0.0 && n.is_finite() => format!("{}", *n as i64),
        other => other.display(),
    }
}

pub(crate) fn values_equal(a: &Value, b: &Value) -> bool {
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

fn parse_num(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(&rest.replace('_', ""), 16).ok().map(|n| n as f64)
    } else if let Some(rest) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        i64::from_str_radix(&rest.replace('_', ""), 2).ok().map(|n| n as f64)
    } else if let Some(rest) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
        i64::from_str_radix(&rest.replace('_', ""), 8).ok().map(|n| n as f64)
    } else {
        s.replace('_', "").parse::<f64>().ok()
    }
}