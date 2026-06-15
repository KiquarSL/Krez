use crate::parser::ast::Stmt;
use crate::session::{Session, source::FileId};
pub mod qbe;

pub enum BackendOutput {
    Text(String),
    Binary(Vec<u8>),
}

impl BackendOutput {
    pub fn write(&self, path: &std::path::Path) -> std::io::Result<()> {
        match self {
            BackendOutput::Text(text) => std::fs::write(path, text),
            BackendOutput::Binary(bin) => std::fs::write(path, bin),
        }
    }
}

pub trait Backend {
    fn compile(&mut self, file_id: FileId, ast: &[Stmt]) -> BackendOutput;
    fn ext(&self) -> String;
}
