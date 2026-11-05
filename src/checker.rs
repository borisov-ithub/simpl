use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::*;
use crate::messages as msg;

#[derive(Clone)]
struct FnInfo {
    required: usize,
    total: usize,
    has_star: bool,
}

#[derive(Clone)]
enum SymbolKind {
    Var,
    Fn(FnInfo),
    Builtin,
}

#[derive(Clone)]
struct Symbol {
    mutable: bool,
    kind: SymbolKind,
}

struct Scope {
    vars: HashMap<String, Symbol>,
    parent: Option<usize>,
}

pub struct Checker {
    scopes: Vec<Scope>,
    current: usize,
    loop_depth: usize,
    current_file: Option<PathBuf>,
    visited_imports: Rc<RefCell<HashSet<PathBuf>>>,
    import_stack: Rc<RefCell<Vec<PathBuf>>>,
}

type CResult<T> = Result<T, String>;

impl Checker {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope { vars: HashMap::new(), parent: None }],
            current: 0,
            loop_depth: 0,
            current_file: None,
            visited_imports: Rc::new(RefCell::new(HashSet::new())),
            import_stack: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn with_file(mut self, file: Option<PathBuf>) -> Self {
        self.current_file = file;
        self
    }

    pub fn check_program(&mut self, program: &Block) -> CResult<()> {
        self.define_builtins();
        for e in program {
            self.check_expr(e)?;
        }
        Ok(())
    }

    fn define_builtins(&mut self) {
        for (name, _) in BUILTINS {
            self.scopes[self.current].vars.insert(
                name.to_string(),
                Symbol { mutable: false, kind: SymbolKind::Builtin },
            );
        }
        // глобальная переменная argv
        self.scopes[self.current].vars.insert(
            "argv".to_string(),
            Symbol { mutable: false, kind: SymbolKind::Var },
        );
        // модули всегда доступны как Var (string.upper, struct.values, ...)
        for module in ["string", "math", "tuple", "struct", "iter", "file", "datetime"] {
            self.scopes[self.current].vars.insert(
                module.to_string(),
                Symbol { mutable: false, kind: SymbolKind::Var },
            );
        }
    }

    // ---------- области ----------
    fn push_scope(&mut self) {
        let parent = self.current;
        self.scopes.push(Scope { vars: HashMap::new(), parent: Some(parent) });
        self.current = self.scopes.len() - 1;
    }

    fn pop_scope(&mut self) {
        if let Some(p) = self.scopes[self.current].parent {
            self.current = p;
        }
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut cur = Some(self.current);
        while let Some(idx) = cur {
            if let Some(sym) = self.scopes[idx].vars.get(name) {
                return Some(sym);
            }
            cur = self.scopes[idx].parent;
        }
        None
    }

    fn define(&mut self, name: &str, mutable: bool, kind: SymbolKind) -> CResult<()> {
        if self.scopes[self.current].vars.contains_key(name) {
            return Err(msg::check_var_already_defined(name));
        }
        self.scopes[self.current].vars.insert(
            name.to_string(),
            Symbol { mutable, kind },
        );
        Ok(())
    }

    fn define_var(&mut self, name: &str, mutable: bool) -> CResult<()> {
        self.define(name, mutable, SymbolKind::Var)
    }

    fn fn_info_from_params(params: &[Param]) -> FnInfo {
        let required = params.iter().filter(|p| !p.star && p.default.is_none()).count();
        let total = params.iter().filter(|p| !p.star).count();
        let has_star = params.iter().any(|p| p.star);
        FnInfo { required, total, has_star }
    }

    // ---------- проверки ----------
    fn check_expr(&mut self, e: &Expr) -> CResult<()> {
        match e {
            Expr::Number(_) | Expr::Str(_) | Expr::Bool(_) | Expr::Nil => Ok(()),

            Expr::Interpolated(parts) => {
                for p in parts {
                    if let StrPart::Expr(e) = p {
                        self.check_expr(e)?;
                    }
                }
                Ok(())
            }

            Expr::Var(name) => {
                if self.lookup(name).is_none() {
                    return Err(msg::check_var_undefined(name));
                }
                Ok(())
            }

            Expr::Tuple(elems) => {
                for e in elems { self.check_expr(e)?; }
                Ok(())
            }

            Expr::Struct(fields) => {
                for f in fields { self.check_expr(&f.value)?; }
                Ok(())
            }

            Expr::StructComprehension { key, value, vars, iter, condition } => {
                self.check_expr(iter)?;
                self.push_scope();
                let res = (|| -> CResult<()> {
                    for v in vars { self.define_var(v, false)?; }
                    if let Some(c) = condition { self.check_expr(c)?; }
                    self.check_expr(key)?;
                    self.check_expr(value)?;
                    Ok(())
                })();
                self.pop_scope();
                res
            }

            Expr::Let { name, mutable, value } => {
                self.check_expr(value)?;
                let kind = match value.as_ref() {
                    Expr::Function { params, .. } | Expr::Lambda { params, .. } => {
                        SymbolKind::Fn(Self::fn_info_from_params(params))
                    }
                    _ => SymbolKind::Var,
                };
                self.define(name, *mutable, kind)?;
                Ok(())
            }

            Expr::LetTuple { names, value } => {
                self.check_expr(value)?;
                for (n, m) in names {
                    self.define_var(n, *m)?;
                }
                Ok(())
            }

            Expr::Assign { name, value } => {
                self.check_expr(value)?;
                match self.lookup(name) {
                    None => Err(msg::check_assign_undefined(name)),
                    Some(sym) if !sym.mutable => Err(msg::check_assign_immutable(name)),
                    _ => Ok(()),
                }
            }

            Expr::AssignField { target, key, value } => {
                self.check_expr(target)?;
                if let AssignKey::Index(idx) = key {
                    self.check_expr(idx)?;
                }
                self.check_expr(value)?;
                Ok(())
            }

            Expr::Unary { operand, .. } | Expr::TypeOf(operand) => self.check_expr(operand),

            Expr::Binary { left, right, .. } => {
                self.check_expr(left)?;
                self.check_expr(right)
            }

            Expr::Pipe { left, right } => {
                self.check_expr(left)?;
                self.check_expr(right)
            }

            Expr::Compose { left, right } => {
                self.check_expr(left)?;
                self.check_expr(right)?;
                if let Some(t) = literal_non_function(left) {
                    return Err(msg::check_compose_non_function(t));
                }
                if let Some(t) = literal_non_function(right) {
                    return Err(msg::check_compose_non_function(t));
                }
                Ok(())
            }

            Expr::Index { target, index } => {
                self.check_expr(target)?;
                self.check_expr(index)?;
                if let Expr::Tuple(elems) = target.as_ref() {
                    if let Expr::Number(n) = index.as_ref() {
                        if n.fract() == 0.0 && *n >= 0.0 {
                            let i = *n as usize;
                            if i >= elems.len() {
                                return Err(msg::check_index_out_of_bounds(i, elems.len()));
                            }
                        }
                    }
                }
                Ok(())
            }

            Expr::Field { target, .. } => self.check_expr(target),

            Expr::If { branches, else_branch } => {
                for (cond, body) in branches {
                    self.check_expr(cond)?;
                    self.push_scope();
                    let res = self.check_block(body);
                    self.pop_scope();
                    res?;
                }
                if let Some(b) = else_branch {
                    self.push_scope();
                    let res = self.check_block(b);
                    self.pop_scope();
                    res?;
                }
                Ok(())
            }

            Expr::ForNum { var, from, to, step, body } => {
                self.check_expr(from)?;
                self.check_expr(to)?;
                if let Some(s) = step {
                    self.check_expr(s)?;
                    if let Expr::Number(n) = s.as_ref() {
                        if *n == 0.0 {
                            return Err(msg::check_for_step_zero());
                        }
                    }
                }
                self.push_scope();
                self.define_var(var, false)?;
                self.loop_depth += 1;
                let res = self.check_block(body);
                self.loop_depth -= 1;
                self.pop_scope();
                res
            }

            Expr::ForIn { vars, iter, body } => {
                self.check_expr(iter)?;
                self.push_scope();
                let res = (|| -> CResult<()> {
                    for v in vars { self.define_var(v, false)?; }
                    self.loop_depth += 1;
                    let r = self.check_block(body);
                    self.loop_depth -= 1;
                    r
                })();
                self.pop_scope();
                res
            }

            Expr::Loop { body } => {
                self.push_scope();
                self.loop_depth += 1;
                let res = self.check_block(body);
                self.loop_depth -= 1;
                self.pop_scope();
                res
            }

            Expr::Break => {
                if self.loop_depth == 0 {
                    return Err(msg::check_break_outside_loop());
                }
                Ok(())
            }

            Expr::Continue => {
                if self.loop_depth == 0 {
                    return Err(msg::check_continue_outside_loop());
                }
                Ok(())
            }

            Expr::Function { params, body } => {
                self.push_scope();
                let res = (|| -> CResult<()> {
                    for p in params { self.define_var(&p.name, false)?; }
                    self.check_block(body)
                })();
                self.pop_scope();
                res
            }

            Expr::Lambda { params, body } => {
                self.push_scope();
                let res = (|| -> CResult<()> {
                    for p in params { self.define_var(&p.name, false)?; }
                    self.check_block(body)
                })();
                self.pop_scope();
                res
            }

            Expr::Call { callee, args } => {
                // если callee — Var, не проверяем check_expr, а сразу check_call
                if !matches!(callee.as_ref(), Expr::Var(_)) {
                    self.check_expr(callee)?;
                }
                for a in args { self.check_expr(a)?; }
                self.check_call(callee, args)
            }

            Expr::Match { subject, arms } => {
                self.check_expr(subject)?;
                for arm in arms {
                    self.push_scope();
                    let res = (|| -> CResult<()> {
                        self.define_pattern(&arm.pattern)?;
                        if let Some(g) = &arm.guard { self.check_expr(g)?; }
                        self.check_block(&arm.body)
                    })();
                    self.pop_scope();
                    res?;
                }
                Ok(())
            }

            Expr::Echo { args, .. } => {
                for a in args { self.check_expr(a)?; }
                Ok(())
            }

            Expr::Return(e) => {
                if let Some(e) = e { self.check_expr(e)?; }
                Ok(())
            }

            Expr::Use { module, alias, names } => self.check_use(module, alias, names),

            Expr::Import { path, alias, names } => self.check_import(path, alias, names),
        }
    }

    fn check_call(&mut self, callee: &Expr, args: &[Expr]) -> CResult<()> {
        match callee {
            Expr::Var(name) => {
                let sym = match self.lookup(name) {
                    Some(s) => s.clone(),
                    None => return Err(msg::check_bi_unknown(name)),
                };
                match &sym.kind {
                    SymbolKind::Builtin => {
                        let (req, tot, star) = builtin_arity(name).unwrap();
                        let got = args.len();
                        if !star && got > tot {
                            return Err(msg::check_bi_arity(name, tot, got));
                        }
                        if got < req {
                            return Err(msg::check_bi_arity(name, req, got));
                        }
                        if name == "flip" && got == 1 {
                            match &args[0] {
                                Expr::Var(fname) => {
                                    if let Some(fsym) = self.lookup(fname) {
                                        match &fsym.kind {
                                            SymbolKind::Builtin => {
                                                return Err(msg::check_flip_builtin());
                                            }
                                            SymbolKind::Fn(fi) => {
                                                if fi.required != 2 {
                                                    return Err(msg::check_flip_needs_binary(fi.required));
                                                }
                                            }
                                            SymbolKind::Var => {}
                                        }
                                    }
                                }
                                Expr::Number(_) | Expr::Str(_) | Expr::Bool(_) | Expr::Nil
                                | Expr::Tuple(_) | Expr::Struct(_) => {
                                    return Err(msg::check_flip_non_function(&literal_type(&args[0])));
                                }
                                _ => {}
                            }
                        }
                        Ok(())
                    }
                    SymbolKind::Fn(fi) => {
                        if !fi.has_star && args.len() > fi.total {
                            return Err(msg::check_too_many_args(fi.total, args.len()));
                        }
                        Ok(())
                    }
                    SymbolKind::Var => Ok(()),
                }
            }
            Expr::Number(_) | Expr::Str(_) | Expr::Bool(_) | Expr::Nil
            | Expr::Tuple(_) | Expr::Struct(_) => {
                Err(msg::check_call_non_function(&literal_type(callee)))
            }
            _ => Ok(()),
        }
    }

    fn check_block(&mut self, block: &Block) -> CResult<()> {
        for e in block { self.check_expr(e)?; }
        Ok(())
    }

    fn define_pattern(&mut self, p: &Pattern) -> CResult<()> {
        match p {
            Pattern::Wildcard => Ok(()),
            Pattern::Var(name) => self.define_var(name, false),
            Pattern::Number(_) | Pattern::Str(_) | Pattern::Bool(_) | Pattern::Nil => Ok(()),
            Pattern::Tuple(ps) => {
                for p in ps { self.define_pattern(p)?; }
                Ok(())
            }
        }
    }

    fn check_use(&mut self, module: &str, alias: &Option<String>, names: &UseNames) -> CResult<()> {
        let fns = match module_fns(module) {
            Some(f) => f,
            None => return Err(msg::check_use_unknown_module(module)),
        };
        // модуль всегда доступен
        self.scopes[0].vars.entry(module.to_string()).or_insert(
            Symbol { mutable: false, kind: SymbolKind::Var },
        );
        match names {
            UseNames::Wildcard => {
                for f in fns {
                    self.scopes[0].vars.entry(f.to_string()).or_insert(
                        Symbol { mutable: false, kind: SymbolKind::Builtin },
                    );
                }
                if let Some(a) = alias {
                    self.scopes[0].vars.entry(a.clone()).or_insert(
                        Symbol { mutable: false, kind: SymbolKind::Var },
                    );
                }
                Ok(())
            }
            UseNames::AliasOnly => {
                if let Some(a) = alias {
                    self.scopes[0].vars.insert(
                        a.clone(),
                        Symbol { mutable: false, kind: SymbolKind::Var },
                    );
                }
                Ok(())
            }
            UseNames::Explicit(list) => {
                for n in list {
                    if !fns.contains(&n.as_str()) {
                        return Err(msg::check_use_name_not_in_module(n, module));
                    }
                    self.scopes[0].vars.insert(
                        n.clone(),
                        Symbol { mutable: false, kind: SymbolKind::Builtin },
                    );
                }
                if let Some(a) = alias {
                    self.scopes[0].vars.entry(a.clone()).or_insert(
                        Symbol { mutable: false, kind: SymbolKind::Var },
                    );
                }
                Ok(())
            }
        }
    }

    fn check_import(&mut self, path: &str, alias: &Option<String>, names: &UseNames) -> CResult<()> {
        let full = match self.resolve_import(path) {
            Ok(p) => p,
            Err(_) => return Ok(()),
        };
        if self.import_stack.borrow().contains(&full) {
            return Err(msg::check_import_cycle(&full.display().to_string()));
        }
        if self.visited_imports.borrow().contains(&full) {
            // уже проверен — но имена нужно зарегистрировать
            // читаем ещё раз, чтобы собрать имена
            if let Ok(src) = std::fs::read_to_string(&full) {
                if let Ok(program) = parse_str(&src) {
                    self.register_import_names(path, alias, names, &program)?;
                }
            }
            return Ok(());
        }
        self.visited_imports.borrow_mut().insert(full.clone());
        self.import_stack.borrow_mut().push(full.clone());
    
        let src = match std::fs::read_to_string(&full) {
            Ok(s) => s,
            Err(_) => {
                self.import_stack.borrow_mut().pop();
                return Ok(());
            }
        };
        let program = match parse_str(&src) {
            Ok(p) => p,
            Err(e) => {
                self.import_stack.borrow_mut().pop();
                return Err(e);
            }
        };
    
        let mut sub = Checker::new().with_file(Some(full.clone()));
        sub.import_stack = self.import_stack.clone();
        sub.visited_imports = self.visited_imports.clone();
        let res = sub.check_program(&program);
        self.import_stack.borrow_mut().pop();
        res?;
    
        self.register_import_names(path, alias, names, &program)?;
        Ok(())
    }
    
    fn register_import_names(
        &mut self,
        path: &str,
        alias: &Option<String>,
        names: &UseNames,
        program: &Block,
    ) -> CResult<()> {
        // собираем top-level let/let_tuple
        let mut all_names: Vec<String> = Vec::new();
        for e in program {
            match e {
                Expr::Let { name, .. } => all_names.push(name.clone()),
                Expr::LetTuple { names, .. } => {
                    for (n, _) in names { all_names.push(n.clone()); }
                }
                _ => {}
            }
        }
    
        let effective_alias = match alias {
            Some(a) => Some(a.clone()),
            None => {
                let name = path
                    .rsplit('/')
                    .next()
                    .unwrap_or(path)
                    .trim_end_matches(".spl")
                    .to_string();
                Some(name)
            }
        };
    
        match names {
            UseNames::AliasOnly => {
                if let Some(a) = effective_alias {
                    self.scopes[0].vars.insert(
                        a,
                        Symbol { mutable: false, kind: SymbolKind::Var },
                    );
                }
            }
            UseNames::Wildcard => {
                for n in &all_names {
                    self.scopes[0].vars.insert(
                        n.clone(),
                        Symbol { mutable: false, kind: SymbolKind::Var },
                    );
                }
                if let Some(a) = effective_alias {
                    self.scopes[0].vars.insert(
                        a,
                        Symbol { mutable: false, kind: SymbolKind::Var },
                    );
                }
            }
            UseNames::Explicit(list) => {
                for n in list {
                    if !all_names.contains(n) {
                        return Err(msg::check_use_name_not_in_module(n, path));
                    }
                    self.scopes[0].vars.insert(
                        n.clone(),
                        Symbol { mutable: false, kind: SymbolKind::Var },
                    );
                }
                if let Some(a) = effective_alias {
                    self.scopes[0].vars.insert(
                        a,
                        Symbol { mutable: false, kind: SymbolKind::Var },
                    );
                }
            }
        }
        Ok(())
    }

    fn resolve_import(&self, path: &str) -> Result<PathBuf, ()> {
        let p = PathBuf::from(path);
        let abs = if p.is_absolute() {
            p
        } else if let Some(base) = &self.current_file {
            base.parent().map(|d| d.join(&p)).unwrap_or(p)
        } else {
            p
        };
        let abs = if abs.extension().is_some() {
            abs
        } else {
            abs.with_extension("spl")
        };
        abs.canonicalize().map_err(|_| ())
    }
}

fn parse_str(src: &str) -> CResult<Block> {
    let mut lexer = crate::lexer::Lexer::new(src);
    let tokens = lexer.tokenize()?;
    let mut parser = crate::parser::Parser::new(tokens);
    parser.parse_program()
}

// ---------- статические таблицы ----------

const BUILTINS: &[(&str, (usize, usize, bool))] = &[
    ("len", (1, 1, false)),
    ("str", (1, 1, false)),
    ("num", (1, 1, false)),
    ("range", (2, 2, false)),
    ("input", (0, 1, false)),
    ("bin", (1, 1, false)),
    ("hex", (1, 1, false)),
    ("oct", (1, 1, false)),
    ("ord", (1, 1, false)),
    ("chr", (1, 1, false)),
    ("sleep", (1, 1, false)),
    ("flip", (1, 1, false)),
];

fn builtin_arity(name: &str) -> Option<(usize, usize, bool)> {
    BUILTINS.iter().find(|(n, _)| *n == name).map(|(_, a)| *a)
}

fn module_fns(module: &str) -> Option<&'static [&'static str]> {
    Some(match module {
        "string" => &["upper", "lower", "trim", "ltrim", "rtrim", "capitalize",
                      "swap_case", "title", "rep", "sub", "find", "count", "replace",
                      "split", "join", "starts_with", "ends_with", "chars",
                      "from_tuple", "is_upper", "is_lower", "is_alnum", "is_alpha",
                      "is_numeric", "is_decimal"],
        "math" => &["abs", "floor", "ceil", "round", "sqrt", "log", "min", "max",
                    "random", "randomseed", "degree", "radian"],
        "tuple" => &["get", "slice", "concat", "push", "reverse", "index_of",
                     "contains", "sort", "zip"],
        "struct" => &["keys", "values", "entries", "has", "get", "merge", "remove",
                      "push", "pop", "unshift", "shift", "insert_at", "remove_at",
                      "from_tuple", "from_pairs", "zip"],
        "iter" => &["map", "filter", "reduce"],
        "file" => &["read", "read_lines", "write", "append", "exists", "remove"],
        "datetime" => &["now", "mktime", "format", "timestamp", "from_timestamp"],
        _ => return None,
    })
}

fn literal_non_function(e: &Expr) -> Option<&'static str> {
    match e {
        Expr::Number(_) => Some("number"),
        Expr::Str(_) => Some("string"),
        Expr::Bool(_) => Some("bool"),
        Expr::Nil => Some("nil"),
        Expr::Tuple(_) => Some("tuple"),
        Expr::Struct(_) => Some("struct"),
        _ => None,
    }
}

fn literal_type(e: &Expr) -> &'static str {
    literal_non_function(e).unwrap_or("unknown")
}