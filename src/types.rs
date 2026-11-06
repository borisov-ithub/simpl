use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Bool,
    Nil,
    Any,
    Struct(Option<Box<Type>>, Option<Box<Type>>),  // struct[K, V]
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Union(Vec<Type>),
    Alias(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Number => write!(f, "number"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Nil => write!(f, "nil"),
            Type::Any => write!(f, "any"),
            Type::Struct(None, None) => write!(f, "struct"),
            Type::Struct(Some(k), Some(v)) => write!(f, "struct[{}, {}]", k, v),
            Type::Struct(Some(k), None) => write!(f, "struct[{}]", k),
            Type::Struct(None, Some(_)) => write!(f, "struct"),
            Type::Tuple(items) => {
                write!(f, "tuple[")?;
                for (i, t) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, "]")
            }
            Type::Function(args, ret) => {
                write!(f, "(")?;
                for (i, t) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Union(items) => {
                for (i, t) in items.iter().enumerate() {
                    if i > 0 { write!(f, " | ")?; }
                    write!(f, "{}", t)?;
                }
                Ok(())
            }
            Type::Alias(name) => write!(f, "{}", name),
        }
    }
}

/// Разворачивает alias в конкретный тип. `aliases` — map имя → тип.
/// Возвращает ошибку при циклическом alias.
pub fn resolve_type(t: &Type, aliases: &HashMap<String, Type>) -> Result<Type, String> {
    let mut visited = HashSet::new();
    resolve_type_rec(t, aliases, &mut visited)
}

fn resolve_type_rec(
    t: &Type,
    aliases: &HashMap<String, Type>,
    visited: &mut HashSet<String>,
) -> Result<Type, String> {
    match t {
        Type::Alias(name) => {
            if !visited.insert(name.clone()) {
                return Err(format!("Циклический alias: {}", name));
            }
            let target = aliases
                .get(name)
                .ok_or_else(|| format!("Неизвестный тип: {}", name))?;
            let r = resolve_type_rec(target, aliases, visited)?;
            visited.remove(name);
            Ok(r)
        }
        Type::Struct(k, v) => {
            let k = match k {
                Some(t) => Some(Box::new(resolve_type_rec(t, aliases, visited)?)),
                None => None,
            };
            let v = match v {
                Some(t) => Some(Box::new(resolve_type_rec(t, aliases, visited)?)),
                None => None,
            };
            Ok(Type::Struct(k, v))
        }
        Type::Tuple(items) => {
            let items = items
                .iter()
                .map(|t| resolve_type_rec(t, aliases, visited))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Tuple(items))
        }
        Type::Function(args, ret) => {
            let args = args
                .iter()
                .map(|t| resolve_type_rec(t, aliases, visited))
                .collect::<Result<Vec<_>, _>>()?;
            let ret = Box::new(resolve_type_rec(ret, aliases, visited)?);
            Ok(Type::Function(args, ret))
        }
        Type::Union(items) => {
            let items = items
                .iter()
                .map(|t| resolve_type_rec(t, aliases, visited))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Union(items))
        }
        other => Ok(other.clone()),
    }
}

/// Парсит строку типа. `None` — не удалось.
pub fn parse_type_str(s: &str) -> Option<Type> {
    let mut p = TypeParser::new(s);
    let t = p.parse_union()?;
    p.skip_ws();
    if p.pos < p.chars.len() { return None; }
    Some(t)
}

struct TypeParser {
    chars: Vec<char>,
    pos: usize,
}

impl TypeParser {
    fn new(s: &str) -> Self {
        Self { chars: s.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> { self.chars.get(self.pos).copied() }
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        Some(c)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() { self.pos += 1; } else { break; }
        }
    }

    fn parse_union(&mut self) -> Option<Type> {
        let mut items = vec![self.parse_arrow()?];
        loop {
            self.skip_ws();
            if self.peek() == Some('|') {
                self.advance();
                items.push(self.parse_arrow()?);
            } else {
                break;
            }
        }
        if items.len() == 1 { Some(items.pop().unwrap()) } else { Some(Type::Union(items)) }
    }

    fn parse_arrow(&mut self) -> Option<Type> {
        let left = self.parse_atom()?;
        self.skip_ws();
        if self.peek() == Some('-') && self.chars.get(self.pos + 1) == Some(&'>') {
            self.pos += 2;
            let right = self.parse_arrow()?;
            // Левая часть — аргументы функции
            let args = match left {
                Type::Tuple(items) => items,
                t => vec![t],
            };
            Some(Type::Function(args, Box::new(right)))
        } else {
            Some(left)
        }
    }

    fn parse_atom(&mut self) -> Option<Type> {
        self.skip_ws();
        match self.peek()? {
            '(' => {
                self.advance();
                let t = self.parse_union()?;
                self.skip_ws();
                if self.advance() != Some(')') { return None; }
                Some(t)
            }
            'n' => self.parse_ident("number", Type::Number),
            's' => {
                self.skip_ws();
                if self.starts_with("string") {
                    self.parse_ident("string", Type::String)
                } else if self.starts_with("struct") {
                    self.pos += 6;
                    self.parse_struct_params()
                } else {
                    None
                }
            }
            'b' => self.parse_ident("bool", Type::Bool),
            't' => {
                if self.starts_with("tuple") {
                    self.pos += 5;
                    self.parse_tuple_params()
                } else {
                    self.parse_ident_any()
                }
            }
            'a' => {
                if self.starts_with("any") {
                    self.parse_ident("any", Type::Any)
                } else {
                    self.parse_ident_any()
                }
            }
            'A'..='Z' | '_' => self.parse_ident_any(),
            _ => None,
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        let s_chars: Vec<char> = s.chars().collect();
        if self.pos + s_chars.len() > self.chars.len() { return false; }
        for (i, c) in s_chars.iter().enumerate() {
            if self.chars[self.pos + i] != *c { return false; }
        }
        true
    }

    fn parse_ident(&mut self, name: &str, t: Type) -> Option<Type> {
        if self.starts_with(name) {
            self.pos += name.len();
            Some(t)
        } else {
            None
        }
    }

    fn parse_ident_any(&mut self) -> Option<Type> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start { return None; }
        let name: String = self.chars[start..self.pos].iter().collect();
        if name == "nil" { Some(Type::Nil) } else { Some(Type::Alias(name)) }
    }

    fn parse_struct_params(&mut self) -> Option<Type> {
        self.skip_ws();
        if self.peek() != Some('[') {
            return Some(Type::Struct(None, None));
        }
        self.advance(); // [
        let first = self.parse_union()?;
        self.skip_ws();
        if self.peek() == Some(']') {
            self.advance();
            return Some(Type::Struct(Some(Box::new(first.clone())), Some(Box::new(first))));
        }
        if self.advance() != Some(',') { return None; }
        let second = self.parse_union()?;
        self.skip_ws();
        if self.advance() != Some(']') { return None; }
        Some(Type::Struct(Some(Box::new(first)), Some(Box::new(second))))
    }

    fn parse_tuple_params(&mut self) -> Option<Type> {
        self.skip_ws();
        if self.peek() != Some('[') { return None; }
        self.advance();
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.advance();
            return Some(Type::Tuple(items));
        }
        loop {
            items.push(self.parse_union()?);
            self.skip_ws();
            if self.peek() == Some(',') {
                self.advance();
                self.skip_ws();
                if self.peek() == Some(']') { self.advance(); break; }
            } else if self.peek() == Some(']') {
                self.advance();
                break;
            } else {
                return None;
            }
        }
        Some(Type::Tuple(items))
    }
}

/// Парсит комментарии `-- :: name :: type` из исходника.
pub fn parse_type_comments(src: &str) -> HashMap<String, Type> {
    let mut map = HashMap::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("-- ::") {
            if let Some((name, ty)) = rest.split_once("::") {
                let name = name.trim();
                let ty = ty.trim();
                if name.is_empty() || !name.chars().next().unwrap().is_alphabetic() {
                    continue;
                }
                if let Some(t) = parse_type_str(ty) {
                    map.insert(name.to_string(), t);
                }
            }
        }
    }
    map
}