pub mod std;

use crate::compiler::KrezCompilerApi;
use crate::parser::{
    Info,
    ast::{AssignOp, MutKind, Stmt},
    expr::{ArithOp, CompOp, Expr, LogicOp, UnaryOp},
    types::Type,
};

pub trait Visitor {
    type Result;

    fn run(&mut self, api: &mut KrezCompilerApi);

    fn visit_expr(&mut self, expr: &Expr) -> Self::Result;
    fn visit_str(&mut self, s: &str, info: &Info) -> Self::Result;
    fn visit_i32(&mut self, n: i32, info: &Info) -> Self::Result;
    fn visit_f32(&mut self, n: f32, info: &Info) -> Self::Result;
    fn visit_id(&mut self, path: &[String], info: &Info) -> Self::Result;
    fn visit_arith(&mut self, left: &Expr, op: ArithOp, right: &Expr, info: &Info) -> Self::Result;
    fn visit_comp(&mut self, left: &Expr, op: CompOp, right: &Expr, info: &Info) -> Self::Result;
    fn visit_logic(&mut self, left: &Expr, op: LogicOp, right: &Expr, info: &Info) -> Self::Result;
    fn visit_unary(&mut self, op: UnaryOp, right: &Expr, info: &Info) -> Self::Result;
    fn visit_call(&mut self, right: &Expr, args: Vec<Expr>, info: &Info) -> Self::Result;

    fn visit_fn(
        &mut self,
        id: String,
        args: Vec<(String, Type)>,
        ret_type: Option<Type>,
        body: Vec<Stmt>,
    ) -> Self::Result;
    fn visit_declare(
        &mut self,
        mut_kind: MutKind,
        id: String,
        ty: Type,
        value: &Expr,
    ) -> Self::Result;
    fn visit_assign(
        &mut self,
        is_dereference: bool,
        id: &str,
        assign: AssignOp,
        value: Expr,
    ) -> Self::Result;
    fn visit_while(&mut self, cond: &Expr, body: Vec<Stmt>) -> Self::Result;
    fn visit_if_else(&mut self, branches: Vec<(Option<Expr>, Vec<Stmt>)>) -> Self::Result;
    fn visit_return(&mut self, ret: Option<Expr>) -> Self::Result;
    fn visit_break(&mut self);
    fn visit_continue(&mut self);
}

pub trait TypeChecker: Visitor<Result = Type> {}
pub trait Optimizer: Visitor<Result = Vec<Stmt>> {}
pub trait Analyzer: Visitor<Result = ()> {}
