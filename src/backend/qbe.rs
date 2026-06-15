use crate::backend::{Backend, BackendOutput};
use crate::parser::ast::Stmt;
use crate::report::{Diagnostic, Level, Phase};
use crate::session::{Session, source::FileId};
use crate::{diag, span_type};
use qbe::{Function, Linkage, Module, Type};
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
}

impl QbeBackend {
    pub fn new(session: Rc<RefCell<Session>>) -> Self {
        Self {
            session,
            module: Module::new(),
            file_id: 0,
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
            Stmt::Func(is_pub, id, args, ret_ty, body) => {
                let linkable = if id == "main" {
                    Linkage::public()
                } else {
                    // temporary solution
                    Linkage::private()
                };
                // let func = Function::new(linkable, id, arguments, ret_ty.to_qbe(self));
                todo!()
            }
            _ => todo!(),
        }
    }

    fn emit_error(&mut self, diag: Diagnostic) {
        self.session.borrow_mut().emit_error(diag);
    }
}

impl crate::parser::types::Type {
    /// Convert Krez to QBE type
    pub fn to_qbe(&self, backend: &mut QbeBackend) -> Option<Type> {
        match self {
            Self::I32(_) => Some(Type::Word),
            Self::F32(_) => Some(Type::Single),
            Self::Bool(_) => Some(Type::Byte),
            Self::Ptr(..) | Self::Str(_) | Self::Array(..) | Self::Custom(..) => Some(Type::Long),

            Self::Void(_) => None,
            Self::Unknown => {
                emit_error!(
                    backend,
                    self,
                    vec!["Use correct type"],
                    vec![],
                    "Invalid type: {}",
                    self
                );
                None
            }
        }
    }
}
