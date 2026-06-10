pub mod std;

use crate::session::source::{FileId, SourceMap};
use strum::Display;

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
	($id:expr, $token:expr) => {
        $crate::report::Span {
            $id,
            $token.line,
            $token.offset,
            $token.len,
        }
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
        $crate::report::Diagnostic::new($msg, $level, $span, $phase, $notes, $helps)
    };
}

pub struct Span {
    pub id: FileId,
    pub line: usize,
    pub offset: usize,
    pub len: usize,
}

pub struct Help {
    pub message: String,
    pub span: Span,
    pub fixed: String,
}

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

#[derive(PartialEq)]
pub enum Level {
    Error,
    Warn,
}

#[derive(PartialEq, Display)]
pub enum Phase {
    Lexing,
    Parsing,
    TypeChecking,
    CodeGen,
}
