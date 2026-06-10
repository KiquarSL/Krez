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
    ($level:expr, $span:expr, $phase:expr, $notes:expr, $helps:expr, $($msg:tt)*) => {
        $crate::report::Diagnostic::new(format!($($msg)*), $level, $span, $phase, $notes, $helps)
    };
	($level:expr, $span:expr, $phase:expr, $($msg:tt)*) => {
        $crate::diag!($level, $span, $phase, vec![], vec![], $($msg)*)
    };
}

#[macro_export]
macro_rules! help {
    ($span:expr, $fixed:expr, $rm:expr, $($msg:tt)*) => {
        $crate::report::Help::new(format!($($msg)*), $span, $fixed, $rm)
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
    pub remove: bool,
}

impl Help {
    pub fn new(message: String, span: Span, fixed: impl Into<String>, remove: bool) -> Self {
        Self {
            message,
            fixed: fixed.into(),
            span,
            remove,
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
        message: impl Into<String>,
        level: Level,
        span: Span,
        phase: Phase,
        notes: Vec<&str>,
        helps: Vec<Help>,
    ) -> Self {
        let notes = notes.iter().map(|s| s.to_string()).collect();
        Self {
            message: message.into(),
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
