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
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct KrezCompiler<'a> {
    lexer: Box<dyn Lexer>,
    parser: Box<dyn Parser>,
    backend: Box<dyn Backend>,
    session: Rc<RefCell<Session<'a>>>,
    build_dir: String,

    ast: HashMap<FileId, Vec<Stmt>>,
    modules: HashMap<FileId, Module>,
    plugins: Vec<Box<dyn Plugin>>,
}

impl KrezCompiler<'a> {
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
            let session = self.session.borrow();
            if session.has_error() {
                session.show_errors();
                return Ok(());
            }
            let ast = self.parser.parse(tokens, file_id);
            self.ast.insert(file_id, ast);
            let session = self.session.borrow();
            if session.has_error() {
                session.show_errors();
                return Ok(());
            }
        }
        let mut api = KrezCompilerApi {
            session: self.session.clone(),
            ast: &mut self.ast,
            modules: &mut self.modules,
        };
        for plugin in &mut self.plugins {
            plugin.run(&mut api);
        }
        let session = self.session.borrow();
        if session.has_error() {
            session.show_errors();
            return Ok(());
        }
        self.session.borrow_mut().load_modules(&self.modules);
        let root = self.find_root_path();

        let session = self.session.borrow();
        let sources = session.sources();
        for (file_id, ast) in &self.ast {
            let output = self.backend.compile(*file_id, ast);
            if session.has_error() {
                session.show_errors();
                return Ok(());
            }
            let source = &sources[*file_id];
            let source_path = Path::new(&source.name);
            let rel = source_path.strip_prefix(&root).unwrap_or(source_path);
            let out_path = Path::new(&(self.build_dir.clone() + &self.backend.out_dir()))
                .join(rel)
                .with_extension("ssa");
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            output.write(&out_path)?;
        }
        Ok(())
    }

    fn find_root_path(&self) -> PathBuf {
        let session = self.session.borrow();
        let sources = session.sources();
        let mut root = PathBuf::from(&sources[0].name);

        for source in sources {
            let path = Path::new(&source.name);
            while !path.starts_with(&root) {
                root = root.parent().unwrap_or(&root).to_path_buf();
            }
        }
        if root.is_file() {
            root = root.parent().unwrap_or(&root).to_path_buf();
        }

        root
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
    pub session: Rc<RefCell<Session<'a>>>,
    pub modules: &'a mut HashMap<FileId, Module>,
}

impl KrezCompilerApi<'_> {
    pub fn push_mod(&mut self, id: FileId, modu: Module) {
        self.modules.insert(id, modu);
    }
}
