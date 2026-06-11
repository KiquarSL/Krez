use crate::backend::Backend;
use crate::lexer::Lexer;
use crate::parser::{Parser, ast::Stmt};
use crate::session::{
    Session,
    source::{FileId, Source},
};
use crate::visitor::{Analyzer, Optimizer, TypeChecker, Visitor};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct KrezCompiler<'a, O> {
    lexer: Box<dyn Lexer>,
    parser: Box<dyn Parser>,
    backend: Box<dyn Backend<Output = O>>,
    session: Session,
    build_dir: &'a str,

    ast: HashMap<FileId, Vec<Stmt>>,
    analyzers: Vec<Box<dyn Analyzer>>,
    type_checkers: Vec<Box<dyn TypeChecker>>,
    optimizers: Vec<Box<dyn Optimizer>>,
}

impl<'a, O> KrezCompiler<'a, O> {
    pub fn new(
        lexer: Box<dyn Lexer>,
        parser: Box<dyn Parser>,
        backend: Box<dyn Backend<Output = O>>,
        session: Session,
        build_dir: &'a str,
        analyzers: Vec<Box<dyn Analyzer>>,
        type_checkers: Vec<Box<dyn TypeChecker>>,
        optimizers: Vec<Box<dyn Optimizer>>,
    ) -> Self {
        Self {
            lexer,
            backend,
            build_dir,
            parser,
            session,
            analyzers,
            optimizers,
            type_checkers,
            ast: HashMap::new(),
        }
    }

    pub fn compile(&mut self, files: Vec<String>) -> std::io::Result<()> {
        let api = KrezCompilerApi {
            session: &mut self.session,
            ast: &mut self.ast,
        };
        for path in files {
            let text = fs::read_to_string(&path)?;
            let source = Source::new(path, text);
            self.session.push_source(source);
        }
        for (file_id, source) in self.session.sources().iter().enumerate() {
            let tokens = self.lexer.tokenize(file_id);
            if self.session.has_error() {
                self.session.show_errors();
                return Ok(());
            }
            let ast = self.parser.parse(tokens, file_id);
            self.ast.insert(file_id, ast);
            if self.session.has_error() {
                self.session.show_errors();
                return Ok(());
            }
        }
        for analyzer in &self.analyzers {
            analyzer.run(api);
        }
        if self.session.has_error() {
            self.session.show_errors();
            return Ok(());
        }
        for type_checker in &self.type_checkers {
            type_checker.run(api);
        }
        if self.session.has_error() {
            self.session.show_errors();
            return Ok(());
        }
        for optimizer in &self.optimizers {
            optimizer.run(api);
        }
        if self.session.has_error() {
            self.session.show_errors();
            return Ok(());
        }
        for (file_id, ast) in self.ast {
            let output = self.backend.compile(&self.ast[&file_id]);
            if self.session.has_error() {
                self.session.show_errors();
                return Ok(());
            }
            let source = &self.session.sources()[file_id];
            let name = source.name.clone() + &self.backend.ext();
            self.backend
                .write(&output, Path::new(&(self.build_dir.to_owned() + &name)))?;
        }
        Ok(())
    }
}

pub struct KrezCompilerApi<'a> {
    pub ast: &'a mut HashMap<FileId, Vec<Stmt>>,
    pub session: &'a mut Session,
}
