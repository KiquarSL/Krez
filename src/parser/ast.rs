use super::expr::Expr;
use super::types::Type;
use strum::Display;

#[derive(Debug)]
pub enum Stmt {
    Declare(String, Type, Expr),
    Assign(String, AssignOp, Expr),
    While(Expr, Vec<Stmt>),
    Func(String, Vec<(String, Type)>, Option<Type>, Vec<Stmt>),
    Expr(Expr),
}

#[derive(Debug, Default, Display)]
pub enum AssignOp {
    #[default]
    #[strum(to_string = "=")]
    Assign,
    #[strum(to_string = "+=")]
    Plus,
    #[strum(to_string = "-=")]
    Minus,
    #[strum(to_string = "*=")]
    Star,
    #[strum(to_string = "/=")]
    Slash,
}
