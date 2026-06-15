use crate::backend::{Backend, qbe::QbeBackend};
use crate::lexer::{Lexer, std::StdLexer};
use crate::parser::{Parser, ast::Stmt, std::StdParser, types::Type};
use crate::plugin::Plugin;
use crate::report::std::{StdReporter, Verbose};
use crate::session::{
    Session,
    source::{FileId, Source},
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;

pub struct KrezCompiler {
    lexer: Box<dyn Lexer>,
    parser: Box<dyn Parser>,
    backend: Box<dyn Backend>,
    session: Rc<RefCell<Session>>,
    build_dir: String,

    ast: HashMap<FileId, Vec<Stmt>>,
    modules: HashMap<FileId, Module>,
    plugins: Vec<Box<dyn Plugin>>,
}

impl KrezCompiler {
    pub fn new(
        lexer: Box<dyn Lexer>,
        parser: Box<dyn Parser>,
        backend: Box<dyn Backend>,
        session: Rc<RefCell<Session>>,
        build_dir: String,
        plugins: Vec<Box<dyn Plugin>>,
    ) -> Self {
        Self {
            session,
            lexer,
            backend,
            build_dir,
            parser,
            plugins,
            ast: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    pub fn default(build_dir: String, verbose: Verbose) -> Self {
        let reporter = StdReporter::new(verbose);
        let session = Rc::new(RefCell::new(Session::new(Some(Box::new(reporter)))));

        let lexer = StdLexer::new(session.clone());
        let parser = StdParser::new(session.clone());
        let backend = QbeBackend::new(session.clone());

        Self::new(
            Box::new(lexer),
            Box::new(parser),
            Box::new(backend),
            session,
            build_dir,
            vec![],
        )
    }

    pub fn compile(&mut self, files: Vec<String>) -> std::io::Result<()> {
        for path in files {
            let text = fs::read_to_string(&path)?;
            let source = Source::new(path, text);
            self.session.borrow_mut().push_source(source);
        }
        for (file_id, _source) in self.session.borrow().sources().iter().enumerate() {
            let tokens = self.lexer.tokenize(file_id);
            self.modules.insert(
                file_id,
                Module {
                    id: file_id.to_string(),
                    pub_func: vec![],
                    pub_uses: vec![],
                },
            );
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
        for plugin in &mut self.plugins {
            plugin.run(&mut api);
        }
        if self.session.borrow().has_error() {
            self.session.borrow().show_errors();
            return Ok(());
        }
        let sess = &self.session.borrow();
        let sources = sess.sources();
        for (file_id, ast) in &self.ast {
            let output = self.backend.compile(*file_id, ast);
            if self.session.borrow().has_error() {
                self.session.borrow().show_errors();
                return Ok(());
            }
            let source = &sources[*file_id];
            let name = source.name.clone() + &self.backend.ext();
            let path = &(self.build_dir.clone() + &name);
            output.write(Path::new(path))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Module {
    pub id: String,
    pub pub_func: Vec<FuncInfo>,
    pub pub_uses: Vec<String>,
}

#[derive(Debug)]
pub struct FuncInfo {
    /// Original function identificator
    pub id: String,
    /// Mangled function identificator. Example: f1, f2, f(n)...
    pub id_mangled: String,
    /// Argument types
    pub args: Vec<Type>,
    /// Return type: Type or None
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
