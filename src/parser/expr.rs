use super::Info;
use strum::Display;

pub type BExpr = Box<Expr>;

/// Using for keeping expressions for AST
/// Keep value and informarion of position
#[derive(Debug, Clone)]
pub enum Expr {
    Invalid,
    Id(Vec<String>, Info),
    Int(i64, Info),
    Float(f64, Info),
    Bool(bool, Info),
    Str(String, Info),
    Arith(BExpr, ArithOp, BExpr, Info),
    Comp(BExpr, CompOp, BExpr, Info),
    Logic(BExpr, LogicOp, BExpr, Info),
    Unary(UnaryOp, BExpr, Info),
    Call(BExpr, Vec<Expr>, Info),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Id(path, _info) => write!(
                f,
                "{}",
                path.iter()
                    .map(|s| s.clone())
                    .collect::<Vec<_>>()
                    .join("::")
            ),
            Expr::Int(n, _info) => write!(f, "{}", n),
            Expr::Float(n, _info) => write!(f, "{}", n),
            Expr::Bool(b, _info) => write!(f, "{}", b),
            Expr::Str(s, _info) => write!(f, "\"{}\"", s),
            Expr::Arith(l, op, r, _info) => write!(f, "({} {} {})", l, op, r),
            Expr::Comp(l, op, r, _info) => write!(f, "({} {} {})", l, op, r),
            Expr::Logic(l, op, r, _info) => write!(f, "({} {} {})", l, op, r),
            Expr::Unary(op, e, _info) => write!(f, "{}{}", op, e),
            Expr::Call(func, args, _info) => {
                write!(
                    f,
                    "{}({})",
                    func,
                    args.iter()
                        .map(|a| format!("{}", a))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::Invalid => write!(f, "INVALID_VALUE"),
        }
    }
}

impl Expr {
    pub fn info(&self) -> Info {
        match self {
            Expr::Str(_, info)
            | Expr::Int(_, info)
            | Expr::Float(_, info)
            | Expr::Id(_, info)
            | Expr::Bool(_, info)
            | Expr::Arith(_, _, _, info)
            | Expr::Comp(_, _, _, info)
            | Expr::Logic(_, _, _, info)
            | Expr::Unary(_, _, info)
            | Expr::Call(_, _, info) => info.clone(),
            Expr::Invalid => Info {
                line: 0,
                offset: 0,
                len: 0,
            },
        }
    }
}

/// Arithmetic operator type
#[derive(Debug, Clone, Display)]
pub enum ArithOp {
    /// +
    #[strum(to_string = "+")]
    Add,
    /// -
    #[strum(to_string = "-")]
    Sub,
    /// *
    #[strum(to_string = "*")]
    Mul,
    /// /
    #[strum(to_string = "/")]
    Div,
}

/// Comparison operator type
#[derive(Debug, Clone, Display)]
pub enum CompOp {
    /// >
    #[strum(to_string = ">")]
    Gt,
    /// >=
    #[strum(to_string = ">=")]
    Ge,
    /// <
    #[strum(to_string = "<")]
    Lt,
    /// <=
    #[strum(to_string = "<=")]
    Le,
    /// ==
    #[strum(to_string = "==")]
    Eq,
    /// !=
    #[strum(to_string = "!=")]
    Ne,
}

/// Logical operator type
#[derive(Debug, Clone, Display)]
pub enum LogicOp {
    /// &&
    #[strum(to_string = "&&")]
    And,
    /// ||
    #[strum(to_string = "||")]
    Or,
}

/// Unary operator type
#[derive(Debug, Clone, Display)]
pub enum UnaryOp {
    /// !
    #[strum(to_string = "!")]
    Not,
    /// -
    #[strum(to_string = "-")]
    Neg,
}
