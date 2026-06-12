use crate::compiler::{FuncInfo, KrezCompilerApi};
use crate::parser::Info;
use crate::parser::ast::{AssignOp, MutKind, Stmt};
use crate::parser::expr::{ArithOp, CompOp, Expr, LogicOp, UnaryOp};
use crate::parser::types::Type;
use crate::visitor::{TypeChecker, Visitor};

pub struct StdCollector {}

impl StdCollector {
    pub fn new() -> Self {
        Self {}
    }
}

impl TypeChecker for StdCollector {}

impl Visitor for StdCollector {
    type Result = Type;

    fn run(&mut self, api: &mut KrezCompilerApi) {
        for (file_id, ast) in &mut *api.ast {
            for stmt in ast {
                if let Stmt::Func(is_pub, id, args, ret_ty, body) = stmt {
                    if !*is_pub {
                        continue;
                    }
                    let mangled = "f".to_owned() + &api.session.new_mangle_func().to_string();
                    let args = args.iter().map(|(_id, ty)| ty.clone()).collect::<Vec<_>>();
                    api.modules
                        .get_mut(file_id)
                        .unwrap()
                        .pub_func
                        .push(FuncInfo::new(id.to_string(), mangled, args, ret_ty.clone()));
                }
            }
        }
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

    fn visit_call(&mut self, right: &Expr, args: Vec<Expr>, _info: &Info) -> Self::Result {
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

    fn visit_while(&mut self, cond: &Expr, body: Vec<Stmt>) -> Self::Result {
        todo!()
    }

    fn visit_if_else(&mut self, branches: Vec<(Option<Expr>, Vec<Stmt>)>) -> Self::Result {
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

    fn visit_break(&mut self) {
        todo!()
    }

    fn visit_continue(&mut self) {
        todo!()
    }
}
