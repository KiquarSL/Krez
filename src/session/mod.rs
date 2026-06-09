pub mod source;

use crate::report::{Diagnostic, Level, Reporter};
use source::SourceMap;

pub struct Session {
    diagnostics: Vec<Diagnostic>,
    source_map: SourceMap,
    reporter: Box<dyn Reporter>,
    has_error: bool,
}

impl Session {
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
