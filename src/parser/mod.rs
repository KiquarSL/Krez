pub mod ast;
pub mod expr;
pub mod types;

use crate::lexer::token::Token;
use ast::Stmt;

pub trait Parser {
    fn parse(&mut self, tokens: &[Token]) -> Vec<Stmt>;
}
