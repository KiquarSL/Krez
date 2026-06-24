use super::expr::Expr;
use super::types::{Type, display_args};
use std::fmt;
use strum;

/// Keeping statement kind and information for compilation
#[derive(Debug)]
pub enum Stmt {
    /** # Fields
    1. Kind mutable (Mut or Fix)
    2. Identificator
    3. Variable data type
    4. Value for assign
    */
    Declare(MutKind, String, Type, Expr),
    /** # Fields
    1. Is dereference (*name)
    2. Identificator
    3. Assign operation type
    4. Value for assign
    */
    Assign(bool, String, AssignOp, Expr),
    /** # Fields
    1. Condition
    2. Body
    */
    While(Expr, Vec<Stmt>),
    /** # Fields
    1. Is export (Visibility in linker)
    2. Is public (Visibility in other modules)
    3. Identifucator
    4. Arguments (Identificator, Type)
    5. Return type
    6. Body
    */
    Func(
        bool,
        bool,
        String,
        Vec<(String, Type)>,
        Option<Type>,
        Vec<Stmt>,
    ),
    /** Keep vector of condition and body
    If condition is None, its else block
    */
    IfElse(Vec<(Option<Expr>, Vec<Stmt>)>),
    /** # Fields
    1. Is public use
    3. Pathes (a::b::c, a::b::d)
    */
    Use(bool, Vec<Vec<String>>),
    /// Keep extern type
    Extern(Vec<Extern>),
    /// Keep return value of None
    Return(Option<Expr>),
    /// Call functions and other expressions
    Expr(Expr),
    /// Break current loop
    Break,
    /// Continue current loop
    Continue,
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
            Stmt::Func(is_export, is_pub, id, args, ret, body) => {
                let pub_str = if *is_export { "pub " } else { "" };
                let exp_str = if *is_pub { "export " } else { "" };
                let head = format!(
                    "{exp_str}{pub_str}fn {id}({}) {}{{",
                    display_args(args.to_vec()),
                    if let Some(ty) = ret {
                        ty.to_string()
                    } else {
                        "".to_string()
                    }
                );
                let body = body
                    .iter()
                    .map(|stmt| format!("    {stmt}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{head}\n{body}\n}}")
            }
            Stmt::IfElse(branches) => {
                let mut buffer = String::new();
                for (i, (cond, body)) in branches.iter().enumerate() {
                    let if_kw = if i == 0 {
                        "if"
                    } else if cond.is_some() {
                        "elif"
                    } else {
                        "else"
                    };
                    let cond_str = if let Some(condition) = cond {
                        condition.clone().to_string()
                    } else {
                        String::from("")
                    } + " ";
                    buffer.push_str(&format!("{if_kw} {cond_str}{{\n"));
                    buffer.push_str(
                        &body
                            .iter()
                            .map(|stmt| format!("    {stmt}"))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                    if i == branches.len() - 1 {
                        buffer.push_str("\n}\n");
                    } else {
                        buffer.push_str("\n} ");
                    }
                }
                buffer
            }
            Stmt::Return(ret) => match ret {
                Some(val) => format!("ret {val};"),
                None => "ret;".to_string(),
            },
            Stmt::Break => String::from("break;"),
            Stmt::Continue => String::from("continue;"),
            Stmt::Use(is_pub, items) => format!(
                "{}",
                items
                    .iter()
                    .map(|i| format!(
                        "{}use {};",
                        if *is_pub { "pub " } else { "" },
                        i.iter()
                            .map(|it| it.to_string())
                            .collect::<Vec<_>>()
                            .join("::")
                    ))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            _ => todo!(),
        };
        write!(f, "{s}")
    }
}

#[derive(Debug)]
pub enum Extern {
    Func(String, Vec<(String, Type)>, Option<Type>),
}

/// Variable muttable type
#[derive(Debug, PartialEq, strum::Display)]
pub enum MutKind {
    /// Keyword: mut
    #[strum(to_string = "mut")]
    Mutable,
    /// Keyword: fix
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
