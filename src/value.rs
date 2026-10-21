use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::ast::{Block, Param};

#[derive(Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Str(String),
    Bool(bool),
    Tuple(Rc<RefCell<Vec<Value>>>),
    Struct(Rc<RefCell<HashMap<String, Value>>>),
    Function(Rc<Function>),
    Builtin(&'static str),
}

pub struct Function {
    pub params: Vec<Param>,
    pub body: Block,
    pub env: crate::env::EnvRef,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Tuple(_) => write!(f, "<кортеж>"),
            Value::Struct(_) => write!(f, "<struct>"),
            Value::Function(_) => write!(f, "<функция>"),
            Value::Builtin(n) => write!(f, "<встроенная {}>", n),
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Number(_) => "number",
            Value::Str(_) => "string",
            Value::Bool(_) => "bool",
            Value::Tuple(_) => "tuple",
            Value::Struct(_) => "struct",
            Value::Function(_) | Value::Builtin(_) => "function",
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    pub fn display(&self) -> String {
        match self {
            Value::Nil => "nil".into(),
            Value::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Tuple(t) => {
                let b = t.borrow();
                let parts: Vec<String> = b.iter().map(|v| v.display()).collect();
                format!("({})", parts.join(", "))
            }
            Value::Struct(s) => {
                let b = s.borrow();
                let parts: Vec<String> = b.iter().map(|(k, v)| format!("{} = {}", k, v.display())).collect();
                format!("{{ {} }}", parts.join(", "))
            }
            Value::Function(_) => "<функция>".into(),
            Value::Builtin(n) => format!("<встроенная {}>", n),
        }
    }
}