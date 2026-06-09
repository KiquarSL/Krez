use crate::session::source::{FileId, SourceMap};

pub trait Reporter {
    fn emit(&self, diag: &Diagnostic, source_map: &SourceMap);
}

#[macro_export]
macro_rules! span {
    ($id:expr, $line:expr, $offset:expr, $len:expr) => {
        $crate::report::Span {
            $id,
            $line,
            $offset,
            $len,
        }
    };
}
#[macro_export]
macro_rules! diag {
    ($msg:expr, $level:expr, $span:expr, $phase:expr, $($notes:expr)*) => {
        $crate::report::Diagnostic {
            $msg,
            $level,
            $span,
            $phase,
            $notes,
        }
    };
}

pub struct Span {
    pub id: FileId,
    pub line: usize,
    pub offset: usize,
    pub len: usize,
}

pub struct Diagnostic {
    pub message: String,
    pub level: Level,
    pub span: Span,
    pub phase: Phase,
    pub notes: Vec<Diagnostic>,
}

impl Diagnostic {
    pub fn new(
        message: String,
        level: Level,
        span: Span,
        phase: Phase,
        notes: Vec<Diagnostic>,
    ) -> Self {
        Self {
            message,
            level,
            notes,
            phase,
            span,
        }
    }
}

#[derive(PartialEq)]
pub enum Level {
    Error,
    Warn,
    Note,
    Help,
}

#[derive(PartialEq)]
pub enum Phase {
    Lexing,
    Parsing,
    TypeChecking,
    CodeGen,
}
