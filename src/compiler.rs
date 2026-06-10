use crate::backend::Backend;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::session::{Session, source::Source};
use std::fs;
use std::path::Path;

pub struct KrezCompiler<'a, O> {
    lexer: Box<dyn Lexer>,
    parser: Box<dyn Parser>,
    backend: Box<dyn Backend<Output = O>>,
    session: Session,
    build_dir: &'a str,
}

impl<'a, O> KrezCompiler<'a, O> {
    pub fn new(
        lexer: Box<dyn Lexer>,
        parser: Box<dyn Parser>,
        backend: Box<dyn Backend<Output = O>>,
        session: Session,
        build_dir: &'a str,
    ) -> Self {
        Self {
            lexer,
            backend,
            build_dir,
            parser,
            session,
        }
    }

    pub fn compile(&mut self, files: Vec<String>) -> std::io::Result<()> {
        for path in files {
            let text = fs::read_to_string(&path)?;
            let source = Source::new(path, text);
            self.session.push_source(source);
        }
        for (file_id, source) in self.session.sources().sources.iter().enumerate() {
            let tokens = self.lexer.tokenize(file_id);
            if self.session.has_error() {
                self.session.show_errors();
                return Ok(());
            }
            let ast = self.parser.parse(&tokens);
            if self.session.has_error() {
                self.session.show_errors();
                return Ok(());
            }
            let output = self.backend.compile(&ast);
            if self.session.has_error() {
                self.session.show_errors();
                return Ok(());
            }
            let name = source.name.clone() + &self.backend.ext();
            self.backend
                .write(&output, Path::new(&(self.build_dir.to_owned() + &name)))?;
        }
        Ok(())
    }
}
