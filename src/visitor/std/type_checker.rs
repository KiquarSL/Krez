use crate::compiler::KrezCompilerApi;
use crate::parser::Info;
use crate::parser::ast::{AssignOp, MutKind, Stmt};
use crate::parser::expr::{ArithOp, CompOp, Expr, LogicOp, UnaryOp};
use crate::parser::types::Type;
use crate::visitor::{TypeChecker, Visitor};

pub struct StdTypeChecker {}

impl StdTypeChecker {
    pub fn new() -> Self {
        Self {}
    }
}

impl TypeChecker for StdTypeChecker {}

impl Visitor for StdTypeChecker {
    type Result = Type;

    fn run(&mut self, _api: &mut KrezCompilerApi) {
        todo!()
    }

    fn visit_expr(&mut self, expr: &Expr) -> Self::Result {
        todo!()
    }

    fn visit_i32(&mut self, n: i32, _info: &Info) -> Self::Result {
        todo!()
    }

    fn visit_f32(&mut self, n: f32, _info: &Info) -> Self::Result {
        todo!()
    }

    fn visit_str(&mut self, s: &str, _info: &Info) -> Self::Result {
        todo!()
    }

    fn visit_id(&mut self, path: &[String], _info: &Info) -> Self::Result {
        todo!()
    }

    fn visit_arith(
        &mut self,
        left: &Expr,
        op: ArithOp,
        right: &Expr,
        _info: &Info,
    ) -> Self::Result {
        todo!()
    }

    fn visit_comp(&mut self, left: &Expr, op: CompOp, right: &Expr, _info: &Info) -> Self::Result {
        todo!()
    }

    fn visit_logic(
        &mut self,
        left: &Expr,
        op: LogicOp,
        right: &Expr,
        _info: &Info,
    ) -> Self::Result {
        todo!()
    }

    fn visit_unary(&mut self, op: UnaryOp, right: &Expr, _info: &Info) -> Self::Result {
        todo!()
    }

    fn visit_call(&mut self, right: &Expr, args: &[Expr], _info: &Info) -> Self::Result {
        todo!()
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Result {
        todo!()
    }

    fn visit_declare(
        &mut self,
        mut_kind: MutKind,
        id: String,
        ty: Type,
        value: &Expr,
    ) -> Self::Result {
        todo!()
    }

    fn visit_assign(
        &mut self,
        is_dereference: bool,
        id: &str,
        assign: AssignOp,
        value: Expr,
    ) -> Self::Result {
        todo!()
    }

    fn visit_while(&mut self, cond: &Expr, body: &[Stmt]) -> Self::Result {
        todo!()
    }

    fn visit_if_else(&mut self, branches: &[(Option<Expr>, Vec<Stmt>)]) -> Self::Result {
        todo!()
    }

    fn visit_fn(
        &mut self,
        id: String,
        args: Vec<(String, Type)>,
        ret_type: Option<Type>,
        body: Vec<Stmt>,
    ) -> Self::Result {
        todo!()
    }

    fn visit_return(&mut self, ret: Option<Expr>) -> Self::Result {
        todo!()
    }

    fn visit_break(&mut self) -> Self::Result {
        todo!()
    }

    fn visit_continue(&mut self) -> Self::Result {
        todo!()
    }
}
