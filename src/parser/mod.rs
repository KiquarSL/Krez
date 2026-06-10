pub mod ast;
pub mod expr;
pub mod std;
pub mod types;

pub trait Parser {
    fn parse(
        &mut self,
        tokens: Vec<crate::lexer::token::Token>,
        file_id: crate::session::source::FileId,
    ) -> Vec<ast::Stmt>;
}

#[derive(Debug, Clone)]
pub struct Info {
    pub line: usize,
    pub offset: usize,
    pub len: usize,
}

use core::fmt;

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.line, self.offset, self.len)
    }
}

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
