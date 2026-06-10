pub mod std;

use crate::session::source::{FileId, SourceMap};
use strum::Display;

pub trait Reporter {
    fn emit(&self, diag: &Diagnostic, source_map: &SourceMap);
}

#[macro_export]
macro_rules! span {
    ($id:expr, $line:expr, $offset:expr, $len:expr) => {
        $crate::report::Span::new($id, $line, $offset, $len)
    };
    ($id:expr, $token:expr) => {
        $crate::span!($id, $token.line, $token.offset, $token.len)
    };
}
#[macro_export]
macro_rules! diag {
    ($msg:expr, $level:expr, $span:expr, $phase:expr, $notes:expr, $helps:expr) => {
        $crate::report::Diagnostic::new($msg, $level, $span, $phase, $notes, $helps)
    };
}

#[macro_export]
macro_rules! help {
    ($msg:expr, $span:expr, $fixed:expr) => {
        $crate::report::Help::new($msg, $span, $fixed)
    };
}

#[derive(Debug, Clone)]
pub struct Span {
    pub id: FileId,
    pub line: usize,
    pub offset: usize,
    pub len: usize,
}

impl Span {
    pub fn new(id: FileId, line: usize, offset: usize, len: usize) -> Self {
        Self {
            id,
            line,
            offset,
            len,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Help {
    pub message: String,
    pub span: Span,
    pub fixed: String,
}

impl Help {
    pub fn new(message: String, span: Span, fixed: String) -> Self {
        Self {
            message,
            fixed,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub level: Level,
    pub span: Span,
    pub phase: Phase,
    pub notes: Vec<String>,
    pub helps: Vec<Help>,
}

impl Diagnostic {
    pub fn new(
        message: String,
        level: Level,
        span: Span,
        phase: Phase,
        notes: Vec<String>,
        helps: Vec<Help>,
    ) -> Self {
        Self {
            message,
            level,
            notes,
            phase,
            span,
            helps,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Level {
    Error,
    Warn,
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Phase {
    Lexing,
    Parsing,
    TypeChecking,
    CodeGen,
}
