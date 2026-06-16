use crate::backend::{Backend, BackendOutput};
use crate::parser::{
    ast::{AssignOp, Stmt},
    expr::Expr,
    types,
};
use crate::report::{Diagnostic, Level, Phase};
use crate::session::{Session, source::FileId};
use crate::{diag, span_type};
use qbe::{Block, Function, Instr, Linkage, Module, Type, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

macro_rules! emit_error {
    ($self:expr, $expr:expr,$notes:expr, $helps:expr, $($msg:tt)*) => {
        $self.emit_error(diag!(
            Level::Error,
            span_type!($self.file_id, $expr),
            Phase::Parsing,
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
                let ret_ty = if let Some(ty) = ret_ty {
                    Some(ty.to_qbe(self))
                } else {
                    None
                };
                let mut func = Function::new(linkable, id, self.to_qbe_args(args.to_vec()), ret_ty);
                let block = func.add_block("start");
                for stmt in body {
                    self.gen_body_stmt(&stmt, block);
                }
                self.module.add_function(func);
            }
            _ => todo!(),
        }
    }

    fn gen_body_stmt(&mut self, stmt: &Stmt, block: &mut Block) {
        match stmt {
            Stmt::Declare(mut_kind, id, ty, val) => {
                let instr_alloc = match ty.size(self) {
                    (4, n) => Instr::Alloc4(n as u32),
                    (8, n) => Instr::Alloc8(n as u64),
                    (16, n) => Instr::Alloc16(n),
                    _ => unreachable!(),
                };
                let ty = ty.to_qbe(self);
                block.assign_instr(Value::Temporary(id.clone()), ty, instr_alloc);
                self.push_var(id.clone(), ty);
                self.gen_body_stmt(
                    &Stmt::Assign(false, id.clone(), AssignOp::default(), val),
                    block,
                );
            }
            Stmt::Assign(is_deref, id, assign_op, val) => {
                block.assign_instr(
                    Value::Temporary(id.clone()),
                    self.var_type(id.clone()),
                    self.gen_expr(val.clone()),
                );
            }
            _ => todo!(),
        }
    }

    fn gen_expr(&self, expr: Expr) -> Instr {
        todo!()
    }

    fn emit_error(&mut self, diag: Diagnostic) {
        self.session.borrow_mut().emit_error(diag);
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}_{}", self.label_count);
        self.label_count += 1;
        label
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn new_tmp(&mut self) -> String {
        let tmp = format!("tmp_{}", self.tmp_count);
        self.tmp_count += 1;
        tmp
    }

    fn var_type(&self, id: String) -> Type {
        for scope in &self.scopes {
            for (var_id, ty) in scope {
                if *var_id == id {
                    return ty.clone();
                }
            }
        }
        Type::Zero
    }

    fn has_var(&self, id: String) -> bool {
        for scope in &self.scopes {
            for var_id in scope.keys() {
                if *var_id == id {
                    return true;
                }
            }
        }
        false
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

    fn to_qbe_args(&mut self, args: Vec<(String, types::Type)>) -> Vec<(Type, Value)> {
        let mut new_args = vec![];
        for (id, ty) in args {
            let val = Value::Temporary(id);
            let ty = ty.to_qbe(self);
            new_args.push((ty, val));
        }
        new_args
    }
}

impl types::Type {
    pub fn size(&self, backend: &mut QbeBackend) -> (u8, u128) {
        match self {
            Self::I32(_) => (4, 1),
            Self::F32(_) => (4, 1),
            Self::Bool(_) => (4, 1),
            Self::Ptr(..) | Self::Str(_) | Self::Array(..) | Self::Custom(..) => (8, 1),

            Self::Unknown => {
                emit_error!(
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
    pub fn to_qbe(&self, backend: &mut QbeBackend) -> Type {
        match self {
            Self::I32(_) => Type::Word,
            Self::F32(_) => Type::Single,
            Self::Bool(_) => Type::Byte,
            Self::Ptr(..) | Self::Str(_) | Self::Array(..) | Self::Custom(..) => Type::Long,

            Self::Unknown => {
                emit_error!(
                    backend,
                    self,
                    vec!["Use correct type"],
                    vec![],
                    "Invalid type: {}",
                    self
                );
                Type::Zero
            }
        }
    }
}
