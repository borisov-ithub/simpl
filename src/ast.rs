#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Str(String),
    Interpolated(Vec<StrPart>),
    Bool(bool),
    Nil,

    Var(String),
    Tuple(Vec<Expr>),
    Struct(Vec<StructField>),
    
    Let { name: String, mutable: bool, value: Box<Expr> },
    LetTuple { names: Vec<(String, bool)>, value: Box<Expr> },
    Assign { name: String, value: Box<Expr> },
    AssignField { target: Box<Expr>, key: AssignKey, value: Box<Expr> },
    
    
    Unary { op: UnaryOp, operand: Box<Expr> },
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    TypeOf(Box<Expr>),
    
    Pipe { left: Box<Expr>, right: Box<Expr> },
    Compose { left: Box<Expr>, right: Box<Expr> },

    Index { target: Box<Expr>, index: Box<Expr> },
    Field { target: Box<Expr>, name: String },

    If { branches: Vec<(Expr, Block)>, else_branch: Option<Block> },

    ForNum { var: String, from: Box<Expr>, to: Box<Expr>, step: Option<Box<Expr>>, body: Block },
    ForIn { vars: Vec<String>, iter: Box<Expr>, body: Block },
    Loop { body: Block },
    Break,
    Continue,

    Function { params: Vec<Param>, body: Block },
    Lambda { params: Vec<Param>, body: Block },
    Call { callee: Box<Expr>, args: Vec<Expr> },

    Match { subject: Box<Expr>, arms: Vec<MatchArm> },

    Echo { args: Vec<Expr>, newline: bool },
    Return(Option<Box<Expr>>),

    Use { module: String, alias: Option<String>, names: UseNames },
    Import {
        path: String,
        alias: Option<String>,
        names: UseNames,
    },
    StructComprehension {
        key: Box<Expr>,
        value: Box<Expr>,
        vars: Vec<String>,
        iter: Box<Expr>,
        condition: Option<Box<Expr>>,
    },
    MethodDef {
        target: Box<Expr>,
        name: String,
        params: Vec<Param>,
        body: Block,
    },
}

#[derive(Debug, Clone)]
pub enum UseNames {
    Wildcard,
    Explicit(Vec<String>),
    AliasOnly,
}

#[derive(Debug, Clone)]
pub enum StrPart { Lit(String), Expr(Expr) }

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub star: bool,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct StructField { pub key: Option<String>, pub value: Expr }

#[derive(Debug, Clone)]
pub struct MatchArm { pub pattern: Pattern, pub guard: Option<Expr>, pub body: Block }

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Var(String),
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
    Tuple(Vec<Pattern>),
}

pub type Block = Vec<Expr>;

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp { Neg, Not, BitNot }

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Concat,
    Eq, NotEq, Lt, LtEq, Gt, GtEq,
    And, Or,
    BitAnd, BitOr, BitXor,
    Shl, Shr,
}

#[derive(Debug, Clone)]
pub enum AssignKey {
    Field(String),
    Index(Box<Expr>),
}