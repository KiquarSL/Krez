use crate::backend::{Backend, BackendOutput};
use crate::parser::{
    Info,
    ast::{AssignOp, Stmt},
    expr::{ArithOp, CompOp, Expr, LogicOp, UnaryOp},
    types::Type,
};
use crate::report::{Diagnostic, Level, Phase};
use crate::session::{Session, source::FileId};
use crate::{diag, span_type};
use qbe::{Block, Cmp, Function, Instr, Linkage, Module, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

macro_rules! emit_error_type {
    ($self:expr, $expr:expr,$notes:expr, $helps:expr, $($msg:tt)*) => {
        $self.emit_error(diag!(
            Level::Error,
            span_type!($self.file_id, $expr),
            Phase::CodeGen,
            $notes,
            $helps,
            $($msg)*
         ));
    };
}

macro_rules! emit_error {
    ($self:expr, $expr:expr,$notes:expr, $helps:expr, $($msg:tt)*) => {
        $self.emit_error(diag!(
            Level::Error,
            span!($self.file_id, $expr.line, $expr.offset, $expr.len),
            Phase::CodeGen,
            $notes,
            $helps,
            $($msg)*
         ));
    };
}

pub struct QbeBackend {
    session: Rc<RefCell<Session>>,
    module: Module,

    file_id: FileId,
    label_count: usize,
    tmp_count: usize,

    scopes: Vec<HashMap<String, Type>>,
    functions: HashMap<String, (Option<Type>, Vec<Type>)>,
}

impl QbeBackend {
    pub fn new(session: Rc<RefCell<Session>>) -> Self {
        Self {
            session,
            module: Module::new(),
            file_id: 0,
            label_count: 0,
            tmp_count: 0,
            scopes: vec![],
            functions: HashMap::new(),
        }
    }
}

impl Backend for QbeBackend {
    fn compile(&mut self, file_id: FileId, ast: &[Stmt]) -> BackendOutput {
        self.file_id = file_id;
        for stmt in ast {
            self.gen_stmt(stmt);
        }
        BackendOutput::Text(self.module.to_string())
    }

    fn ext(&self) -> String {
        String::from(".ssa")
    }
}

impl QbeBackend {
    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Func(_is_pub, id, args, ret_ty, body) => {
                let linkable = if id == "main" {
                    Linkage::public()
                } else {
                    // temporary solution
                    Linkage::private()
                };
                self.functions.insert(
                    id.clone(),
                    (
                        ret_ty.clone(),
                        args.iter().map(|(_id, ty)| ty.clone()).collect(),
                    ),
                );
                let ret_ty = if let Some(ty) = ret_ty {
                    Some(ty.to_qbe(self))
                } else {
                    None
                };
                let mut func =
                    Function::new(linkable, id, self.to_qbe_func_args(args.to_vec()), ret_ty);
                let block = func.add_block("start");
                self.push_scope();
                for stmt in body {
                    self.gen_body_stmt(&stmt, block);
                }
                self.pop_scope();
                self.module.add_function(func);
            }
            _ => todo!(),
        }
    }

    fn gen_body_stmt(&mut self, stmt: &Stmt, block: &mut Block) {
        match stmt {
            Stmt::Declare(_mut_kind, id, ty, val) => {
                let instr_alloc = match ty.size(self) {
                    (4, n) => Instr::Alloc4(n as u32),
                    (8, n) => Instr::Alloc8(n as u64),
                    (16, n) => Instr::Alloc16(n),
                    _ => unreachable!(),
                };
                block.assign_instr(Value::Temporary(id.clone()), ty.to_qbe(self), instr_alloc);
                self.push_var(id.clone(), ty.clone());
                self.gen_body_stmt(
                    &Stmt::Assign(false, id.clone(), AssignOp::default(), val.clone()),
                    block,
                );
            }
            Stmt::Assign(_is_deref, id, _assign_op, val) => {
                let ty = self.var_info(id.clone());
                let value = self.gen_expr(val.clone(), block, ty.clone());
                block.assign_instr(
                    Value::Temporary(id.clone()),
                    ty.to_qbe(self),
                    Instr::Copy(value.1),
                );
            }
            _ => todo!(),
        }
    }

    fn gen_expr(&mut self, expr: Expr, block: &mut Block, ty: Type) -> (String, Value, Type) {
        let tmp = self.new_tmp();
        let value = match expr.clone() {
            Expr::Int(n, info) => {
                let value = Value::Const(n as u64);
                (value, Type::I32(info))
            }
            Expr::Float(n, info) => {
                let value = Value::Const(n.to_bits());
                (value, Type::F32(info))
            }
            Expr::Id(id, _info) => {
                let id = id.last().unwrap().clone();
                let value = Value::Temporary(id.clone());
                (value, self.var_info(id))
            }
            Expr::Bool(truth, info) => {
                let truth = if truth { 1 } else { 0 };
                let value = Value::Const(truth);
                (value, Type::Bool(info))
            }
            Expr::Arith(left, op, right, _info) => {
                let (_left_tmp, left_value, left_ty) = self.gen_expr(*left, block, ty.clone());
                let (_right_tmp, right_value, _right_ty) = self.gen_expr(*right, block, ty.clone());
                let instr = match op {
                    ArithOp::Add => Instr::Add(left_value, right_value),
                    ArithOp::Div => Instr::Div(left_value, right_value),
                    ArithOp::Mul => Instr::Mul(left_value, right_value),
                    ArithOp::Sub => Instr::Sub(left_value, right_value),
                };
                let value = Value::Temporary(tmp.clone());
                block.assign_instr(value.clone(), ty.to_qbe(self), instr);
                (value, left_ty)
            }
            Expr::Logic(left, op, right, _info) => {
                let (_left_tmp, left_value, left_ty) = self.gen_expr(*left, block, ty.clone());
                let (_right_tmp, right_value, _right_ty) = self.gen_expr(*right, block, ty.clone());
                let instr = match op {
                    LogicOp::And => Instr::And(left_value, right_value),
                    LogicOp::Or => Instr::Or(left_value, right_value),
                };
                let value = Value::Temporary(tmp.clone());
                block.assign_instr(value.clone(), ty.to_qbe(self), instr);
                (value, left_ty)
            }
            Expr::Comp(left, op, right, info) => {
                let (_left_tmp, left_value, left_ty) = self.gen_expr(*left, block, ty.clone());
                let (_right_tmp, right_value, _right_ty) = self.gen_expr(*right, block, ty.clone());
                let cmp = match op {
                    CompOp::Eq => Cmp::Eq,
                    CompOp::Ne => Cmp::Eq,
                    CompOp::Gt => {
                        if left_ty.is_unsigned() {
                            Cmp::Ugt
                        } else {
                            Cmp::Sgt
                        }
                    }
                    CompOp::Ge => {
                        if left_ty.is_unsigned() {
                            Cmp::Uge
                        } else {
                            Cmp::Sge
                        }
                    }
                    CompOp::Lt => {
                        if left_ty.is_unsigned() {
                            Cmp::Ult
                        } else {
                            Cmp::Slt
                        }
                    }
                    CompOp::Le => {
                        if left_ty.is_unsigned() {
                            Cmp::Ule
                        } else {
                            Cmp::Sle
                        }
                    }
                };
                let instr = Instr::Cmp(qbe::Type::Word, cmp, left_value, right_value);
                let value = Value::Temporary(tmp.clone());
                block.assign_instr(value.clone(), ty.to_qbe(self), instr);
                (value, Type::Bool(info))
            }
            Expr::Unary(op, expr, _info) => {
                let (_left_tmp, value, ty) = self.gen_expr(*expr.clone(), block, ty.clone());
                let instr = match op {
                    UnaryOp::Neg => Instr::Neg(value),
                    UnaryOp::Not => {
                        let (not_tmp, _not_value, _not_ty) = self.gen_expr(
                            Expr::Comp(
                                expr.clone(),
                                CompOp::Ne,
                                Box::new(Expr::Int(0, Info::empty())),
                                Info::empty(),
                            ),
                            block,
                            ty.clone(),
                        );
                        Instr::Copy(Value::Temporary(not_tmp))
                    }
                };
                let value = Value::Temporary(tmp.clone());
                block.assign_instr(value.clone(), ty.to_qbe(self), instr);
                (value, ty)
            }
            Expr::Call(id, args, _info) => {
                let id = match *id {
                    Expr::Id(path, _) => path.last().expect("Path need have last"),
                    _ => unreachable!(),
                };
                let args = self.to_qbe_args(block, id.clone(), args);
                block.add_instr(Instr::Call(id.clone(), args, None));
                (
                    Value::Temporary(tmp.clone()),
                    self.functions.get(id).unwrap().0.clone().unwrap(),
                )
            }
            Expr::Invalid => {
                unreachable!("Invalid values cannot be in backend")
            }
            _ => todo!("{expr}"),
        };
        (tmp, value.0, value.1)
    }

    fn emit_error(&mut self, diag: Diagnostic) {
        self.session.borrow_mut().emit_error(diag);
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}_{}", self.label_count);
        self.label_count += 1;
        label
    }

    fn new_tmp(&mut self) -> String {
        let tmp = format!("_tmp_{}", self.tmp_count);
        self.tmp_count += 1;
        tmp
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn var_info(&self, id: String) -> Type {
        for scope in &self.scopes {
            for (var_id, ty) in scope {
                if *var_id == id {
                    return ty.clone();
                }
            }
        }
        Type::Unknown
    }

    fn push_var(&mut self, id: String, ty: Type) {
        for scope in &mut self.scopes {
            for var_id in scope.keys() {
                if *var_id == id {
                    scope.insert(id, ty);
                    return;
                }
            }
        }
        if let Some(last) = self.scopes.last_mut() {
            last.insert(id, ty);
        }
    }

    fn to_qbe_func_args(&mut self, args: Vec<(String, Type)>) -> Vec<(qbe::Type, Value)> {
        let mut new_args = vec![];
        for (id, ty) in args {
            let val = Value::Temporary(id);
            let ty = ty.to_qbe(self);
            new_args.push((ty, val));
        }
        new_args
    }

    fn to_qbe_args(
        &mut self,
        block: &mut Block,
        id: String,
        args: Vec<Expr>,
    ) -> Vec<(qbe::Type, Value)> {
        let mut new_args = vec![];
        for (i, expr) in args.iter().enumerate() {
            let ty = self.functions.get(&id).unwrap().1[i].clone();
            let value = self.gen_expr(expr.clone(), block, ty.clone());
            new_args.push((ty.to_qbe(self), value.1));
        }
        new_args
    }
}

impl Type {
    /// Get alloc variant type and size: (alloc4 1)
    pub fn size(&self, backend: &mut QbeBackend) -> (u8, u128) {
        match self {
            Self::I32(_) | Self::U32(_) | Self::F32(_) | Self::Bool(_) => (4, 1),
            Self::Ptr(..) | Self::Str(_) | Self::Array(..) | Self::Custom(..) => (8, 1),

            Self::Unknown => {
                emit_error_type!(
                    backend,
                    self,
                    vec!["Use correct type"],
                    vec![],
                    "Invalid type: {}",
                    self
                );
                (4, 1)
            }
        }
    }
    /// Convert Krez to QBE type
    pub fn to_qbe(&self, backend: &mut QbeBackend) -> qbe::Type {
        match self {
            Self::I32(_) | Self::U32(_) | Self::Bool(_) => qbe::Type::Word,
            Self::Ptr(..) | Self::Str(_) | Self::Array(..) | Self::Custom(..) => qbe::Type::Long,
            Self::F32(_) => qbe::Type::Single,
            Self::Unknown => {
                emit_error_type!(
                    backend,
                    self,
                    vec!["Use correct type"],
                    vec![],
                    "Invalid type: {}",
                    self
                );
                qbe::Type::Zero
            }
        }
    }
}
