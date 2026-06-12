use crate::backend::Backend;
use crate::lexer::Lexer;
use crate::parser::{Parser, ast::Stmt, types::Type};
use crate::session::{
    Session,
    source::{FileId, Source},
};
use crate::visitor::{Analyzer, Optimizer, TypeChecker, Visitor};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;

pub struct KrezCompiler<O> {
    lexer: Box<dyn Lexer>,
    parser: Box<dyn Parser>,
    backend: Box<dyn Backend<Output = O>>,
    session: Rc<RefCell<Session>>,
    build_dir: String,

    ast: HashMap<FileId, Vec<Stmt>>,
    modules: HashMap<FileId, Module>,

    analyzers: Vec<Box<dyn Analyzer>>,
    type_checkers: Vec<Box<dyn TypeChecker>>,
    optimizers: Vec<Box<dyn Optimizer>>,
}

impl<O> KrezCompiler<O> {
    pub fn new(
        lexer: Box<dyn Lexer>,
        parser: Box<dyn Parser>,
        backend: Box<dyn Backend<Output = O>>,
        session: Session,
        build_dir: String,
        analyzers: Vec<Box<dyn Analyzer>>,
        type_checkers: Vec<Box<dyn TypeChecker>>,
        optimizers: Vec<Box<dyn Optimizer>>,
    ) -> Self {
        Self {
            session: Rc::new(RefCell::new(session)),
            lexer,
            backend,
            build_dir,
            parser,
            analyzers,
            optimizers,
            type_checkers,
            ast: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    pub fn compile(&mut self, files: Vec<String>) -> std::io::Result<()> {
        for path in files {
            let text = fs::read_to_string(&path)?;
            let source = Source::new(path, text);
            self.session.borrow_mut().push_source(source);
        }
        for (file_id, _source) in self.session.borrow().sources().iter().enumerate() {
            let tokens = self.lexer.tokenize(file_id);
            if self.session.borrow().has_error() {
                self.session.borrow().show_errors();
                return Ok(());
            }
            let ast = self.parser.parse(tokens, file_id);
            self.ast.insert(file_id, ast);
            if self.session.borrow().has_error() {
                self.session.borrow().show_errors();
                return Ok(());
            }
        }
        let mut api = KrezCompilerApi {
            session: &mut self.session.borrow_mut(),
            ast: &mut self.ast,
            modules: &mut self.modules,
        };
        for analyzer in &mut self.analyzers {
            analyzer.run(&mut api);
        }
        if self.session.borrow().has_error() {
            self.session.borrow().show_errors();
            return Ok(());
        }
        for type_checker in &mut self.type_checkers {
            type_checker.run(&mut api);
        }
        if self.session.borrow().has_error() {
            self.session.borrow().show_errors();
            return Ok(());
        }
        for optimizer in &mut self.optimizers {
            optimizer.run(&mut api);
        }
        if self.session.borrow().has_error() {
            self.session.borrow().show_errors();
            return Ok(());
        }
        let session = &self.session.borrow();
        let sources = session.sources();
        for (file_id, _ast) in &self.ast {
            let output = self.backend.compile(&self.ast[&file_id]);
            if self.session.borrow().has_error() {
                self.session.borrow().show_errors();
                return Ok(());
            }
            let source = &sources[*file_id];
            let name = source.name.clone() + &self.backend.ext();
            self.backend
                .write(&output, Path::new(&(self.build_dir.to_owned() + &name)))?;
        }
        Ok(())
    }
}

pub struct Module {
    pub id: String,
    pub pub_func: Vec<FuncInfo>,
}

pub struct FuncInfo {
    /// Original function identificator
    pub id: String,
    /// Mangled function identificator. Example: f1, f2, f(n)...
    pub id_mangled: String,
    pub args: Vec<Type>,
    pub ret_ty: Option<Type>,
}

impl FuncInfo {
    pub fn new(id: String, id_mangled: String, args: Vec<Type>, ret_ty: Option<Type>) -> Self {
        Self {
            id,
            args,
            id_mangled,
            ret_ty,
        }
    }
}

pub struct KrezCompilerApi<'a> {
    pub ast: &'a mut HashMap<FileId, Vec<Stmt>>,
    pub session: &'a mut Session,
    pub modules: &'a mut HashMap<FileId, Module>,
}
