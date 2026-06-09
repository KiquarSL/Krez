use crate::backend::Backend;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::report::Reporter;
use crate::session::Session;

pub struct KrezCompiler<O> {
    lexer: Box<dyn Lexer>,
    parser: Box<dyn Parser>,
    reporter: Box<dyn Reporter>,
    backend: Box<dyn Backend<Output = O>>,
    session: Session,
}

impl<O> KrezCompiler<O> {
    pub fn default() -> Self {
        todo!()
    }
}
