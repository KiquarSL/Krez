pub mod source;

use crate::compiler::Module;
use crate::report::{Diagnostic, Level, Reporter};
use source::{FileId, Source, SourceMap};
use std::collections::HashMap;

pub struct Session<'a> {
    diagnostics: Vec<Diagnostic>,
    source_map: SourceMap,
    reporter: Option<Box<dyn Reporter>>,
    has_error: bool,
    modules: Option<&'a HashMap<FileId, Module>>,

    mangle_func_count: usize,
}

impl<'a> Session<'a> {
    pub fn new(reporter: Option<Box<dyn Reporter>>) -> Self {
        Self {
            reporter,
            source_map: SourceMap { sources: vec![] },
            diagnostics: vec![],
            has_error: false,
            mangle_func_count: 0,
            modules: None,
        }
    }

    pub fn load_modules(&mut self, mods: &'a HashMap<FileId, Module>) {
        self.modules = Some(mods);
    }

    pub fn new_mangle_func(&mut self) -> usize {
        let mangle = self.mangle_func_count;
        self.mangle_func_count += 1;
        mangle
    }

    pub fn push_source(&mut self, source: Source) {
        self.source_map.sources.push(source);
    }

    pub fn sources(&self) -> &Vec<Source> {
        &self.source_map.sources
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    pub fn source_map_mut(&mut self) -> &mut SourceMap {
        &mut self.source_map
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }

    pub fn emit_error(&mut self, diag: Diagnostic) {
        if diag.level == Level::Error {
            self.has_error = true;
        }
        self.diagnostics.push(diag);
    }

    pub fn show_errors(&self) {
        for diag in &self.diagnostics {
            match &self.reporter {
                Some(rep) => rep.emit(diag, &self.source_map),
                None => {}
            }
        }
    }
}
