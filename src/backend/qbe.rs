use crate::backend::{Backend, BackendOutput};
use crate::parser::{ast::Stmt, types};
use crate::report::{Diagnostic, Level, Phase};
use crate::session::{Session, source::FileId};
use crate::{diag, span_type};
use qbe::{Block, Function, Linkage, Module, Type, Value};
use std::cell::RefCell;
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
}

impl QbeBackend {
    pub fn new(session: Rc<RefCell<Session>>) -> Self {
        Self {
            session,
            module: Module::new(),
            file_id: 0,

            label_count: 0,
            tmp_count: 0,
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
                let mut func = Function::new(
                    linkable,
                    id,
                    self.to_qbe_args(args.to_vec()),
                    Some(ret_ty.to_qbe(self)),
                );
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
    /// Convert Krez to QBE type
    pub fn to_qbe(&self, backend: &mut QbeBackend) -> Type {
        match self {
            Self::I32(_) => Type::Word,
            Self::F32(_) => Type::Single,
            Self::Bool(_) => Type::Byte,
            Self::Ptr(..) | Self::Str(_) | Self::Array(..) | Self::Custom(..) => Type::Long,

            Self::Void(_) => Type::Zero,
            Self::Unknown => {
                emit_error!(
                    backend,
                    self,
                    vec!["Use correct type"],
                    vec![],
                    "Invalid type: {}",
                    self
                );
                unreachable!("It checking before compilation and break if found error")
            }
        }
    }
}
