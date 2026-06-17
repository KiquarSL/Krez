pub mod ast;
pub mod expr;
pub mod std;
pub mod types;

pub trait Parser {
    /// Method for build AST (Abstract Syntax Tree) from tokens
    /// Return vector of statements
    /// In implements recommended transmit Session for using source code which you can take with file_id and add errors. See example in crate::parser/std.rs
    fn parse(
        &mut self,
        tokens: Vec<crate::lexer::token::Token>,
        file_id: crate::session::source::FileId,
    ) -> Vec<ast::Stmt>;
}

/// Position information for Expr and Type
/// Usign for build and show errors
#[derive(Debug, Clone)]
pub struct Info {
    pub line: usize,
    pub offset: usize,
    pub len: usize,
}

impl Info {
    pub fn empty() -> Info {
        Info {
            len: 0,
            line: 0,
            offset: 0,
        }
    }
}

use core::fmt;

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} {}", self.line, self.offset, self.len)
    }
}

/// Create Info from position data or Token
#[macro_export]
macro_rules! info {
    ($line:expr, $offset:expr, $len:expr) => {
        $crate::parser::Info {
            line: $line,
            offset: $offset,
            len: $len,
        }
    };
    ($tkn:expr) => {
        info!($tkn.line, $tkn.offset, $tkn.len)
    };
}
