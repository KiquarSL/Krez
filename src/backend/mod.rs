use crate::parser::ast::Stmt;

pub trait Backend {
    type Output;

    fn compile(&mut self, ast: &[Stmt]) -> Result<Self::Output, ()>;
}
