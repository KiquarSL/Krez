use super::expr::Expr;
use super::types::{Type, display_args};
use std::fmt;
use strum;

#[derive(Debug)]
pub enum Stmt {
    Declare(MutKind, String, Type, Expr),
    Assign(bool, String, AssignOp, Expr),
    While(Expr, Vec<Stmt>),
    Func(String, Vec<(String, Type)>, Option<Type>, Vec<Stmt>),
    Expr(Expr),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Stmt::Declare(mut_kind, id, ty, val) => format!("{mut_kind} {id}: {ty} = {val};"),
            Stmt::Assign(is_dereference, id, assign, val) => format!(
                "{}{id} {assign} {val};",
                if *is_dereference { "*" } else { "" }
            ),
            Stmt::Expr(expr) => format!("{expr}",),
            Stmt::While(cond, body) => {
                let head = format!("while {cond} {{");
                let body = body
                    .iter()
                    .map(|stmt| format!("\t{stmt}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{head}\n{body}\n}}")
            }
            Stmt::Func(id, args, ret, body) => {
                let ret_ty = match ret {
                    Some(ty) => ty.to_string(),
                    None => "".to_string(),
                };
                let head = format!("fn {id}({}) {ret_ty} {{", display_args(args.to_vec()),);
                let body = body
                    .iter()
                    .map(|stmt| format!("\t{stmt}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{head}\n{body}\n}}")
            }
         };
        write!(f, "{s}")
    }
}

#[derive(Debug, strum::Display)]
pub enum MutKind {
    #[strum(to_string = "mut")]
    Mutable,
    #[strum(to_string = "fix")]
    Fixed,
}

#[derive(Debug, Default, strum::Display)]
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
