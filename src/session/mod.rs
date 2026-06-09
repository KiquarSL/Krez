pub mod source;

use crate::report::{Diagnostic, Level, Reporter};
use source::{Source, SourceMap};

pub struct Session {
    diagnostics: Vec<Diagnostic>,
    source_map: SourceMap,
    reporter: Box<dyn Reporter>,
    has_error: bool,
}

impl Session {
    pub fn new(reporter: Box<dyn Reporter>) -> Self {
        Self {
            reporter,
            source_map: SourceMap { sources: vec![] },
            diagnostics: vec![],
            has_error: false,
        }
    }

    pub fn push_source(&mut self, source: Source) {
        self.source_map.sources.push(source);
    }

    pub fn sources(&self) -> &SourceMap {
        &self.source_map
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
            self.reporter.emit(diag, &self.source_map);
        }
    }
}
