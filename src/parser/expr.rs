use super::Info;
use strum::Display;

pub type BExpr = Box<Expr>;

#[derive(Debug, Clone)]
pub enum Expr {
    Id(String, Info),

    Int(i64, Info),

    Float(f64, Info),

    Bool(bool, Info),

    Str(String, Info),

    Arith(BExpr, ArithOp, BExpr, Info),

    Comp(BExpr, CompOp, BExpr, Info),

    Logic(BExpr, LogicOp, BExpr, Info),

    Unary(UnaryOp, BExpr, Info),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Id(s, _info) => write!(f, "{}", s),
            Expr::Int(n, _info) => write!(f, "{}", n),
            Expr::Float(n, _info) => write!(f, "{}", n),
            Expr::Bool(b, _info) => write!(f, "{}", b),
            Expr::Str(s, _info) => write!(f, "\"{}\"", s),
            Expr::Arith(l, op, r, _info) => write!(f, "({} {} {})", l, op, r),
            Expr::Comp(l, op, r, _info) => write!(f, "({} {} {})", l, op, r),
            Expr::Logic(l, op, r, _info) => write!(f, "({} {} {})", l, op, r),
            Expr::Unary(op, e, _info) => write!(f, "{}{}", op, e),
        }
    }
}

impl Expr {
    pub fn info(&self) -> Info {
        match self {
            Expr::Str(_, info) => info.clone(),
            Expr::Int(_, info) => info.clone(),
            Expr::Float(_, info) => info.clone(),
            Expr::Id(_, info) => info.clone(),
            Expr::Bool(_, info) => info.clone(),
            Expr::Arith(_, _, _, info) => info.clone(),
            Expr::Comp(_, _, _, info) => info.clone(),
            Expr::Logic(_, _, _, info) => info.clone(),
            Expr::Unary(_, _, info) => info.clone(),
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum ArithOp {
    #[strum(to_string = "+")]
    Add,
    #[strum(to_string = "-")]
    Sub,
    #[strum(to_string = "*")]
    Mul,
    #[strum(to_string = "/")]
    Div,
}

#[derive(Debug, Clone, Display)]
pub enum CompOp {
    #[strum(to_string = ">")]
    Gt,
    #[strum(to_string = ">=")]
    Ge,
    #[strum(to_string = "<")]
    Lt,
    #[strum(to_string = "<=")]
    Le,
    #[strum(to_string = "==")]
    Eq,
    #[strum(to_string = "!=")]
    Ne,
}

#[derive(Debug, Clone, Display)]
pub enum LogicOp {
    #[strum(to_string = "&&")]
    And,
    #[strum(to_string = "||")]
    Or,
}

#[derive(Debug, Clone, Display)]
pub enum UnaryOp {
    #[strum(to_string = "!")]
    Not,
    #[strum(to_string = "-")]
    Neg,
}
