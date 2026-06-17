use crate::parser::ast::Stmt;
use crate::session::source::FileId;
use std::fmt;
pub mod qbe;

#[derive(Debug, Clone)]
pub enum BackendOutput {
    Text(String),
    Binary(Vec<u8>),
}

impl fmt::Display for BackendOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Text(txt) => txt.clone(),
                Self::Binary(bytes) => format!("{:?}", bytes),
            }
        )
    }
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
    fn out_dir(&self) -> String;
}
