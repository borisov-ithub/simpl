use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::messages as msg;
use crate::value::Value;

pub type EnvRef = Rc<RefCell<Env>>;

pub struct Env {
    pub vars: HashMap<String, (Value, bool)>,
    pub parent: Option<EnvRef>,
}

impl Env {
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Self { vars: HashMap::new(), parent: None }))
    }

    pub fn child(parent: &EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self { vars: HashMap::new(), parent: Some(parent.clone()) }))
    }

    pub fn define(&mut self, name: &str, value: Value, mutable: bool) {
        self.vars.insert(name.to_string(), (value, mutable));
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some((v, _)) = self.vars.get(name) { return Some(v.clone()); }
        if let Some(p) = &self.parent { return p.borrow().get(name); }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some((v, m)) = self.vars.get_mut(name) {
            if !*m {
                return Err(msg::env_assign_immutable(name));
            }
            *v = value;
            return Ok(());
        }
        if let Some(p) = &self.parent {
            return p.borrow_mut().assign(name, value);
        }
        Err(msg::env_assign_undefined(name))
    }
}