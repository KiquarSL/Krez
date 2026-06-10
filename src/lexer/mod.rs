pub mod std;
pub mod token;

use crate::session::source::FileId;
use token::Token;

pub trait Lexer {
    fn tokenize(&mut self, file_id: FileId) -> Vec<Token>;
}
