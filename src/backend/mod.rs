use crate::parser::ast::Stmt;

pub trait Backend {
    type Output;

    fn compile(&mut self, ast: &[Stmt]) -> Self::Output;
    fn ext(&self) -> String;
    fn write(&self, out: &Self::Output, path: &std::path::Path) -> std::io::Result<()>;
}
