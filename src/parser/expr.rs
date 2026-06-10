use super::Info;
use strum::Display;

pub type BExpr = Box<Expr>;

#[derive(Debug, Clone, Display)]
pub enum Expr {
    #[strum(to_string = "{0}")]
    Id(String, Info),
    #[strum(to_string = "{0}")]
    Int(i64, Info),
    #[strum(to_string = "{0}")]
    Float(f64, Info),
    #[strum(to_string = "{0}")]
    Bool(bool, Info),
    #[strum(to_string = "\"{0}\"")]
    Str(String, Info),

    #[strum(to_string = "({0} {1} {2})")]
    Arith(BExpr, ArithOp, BExpr, Info),

    #[strum(to_string = "({0} {1} {2})")]
    Comp(BExpr, CompOp, BExpr, Info),

    #[strum(to_string = "({0} {1} {2})")]
    Logic(BExpr, LogicOp, BExpr, Info),
    #[strum(to_string = "{0}{1}")]
    Unary(UnaryOp, BExpr, Info),
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
