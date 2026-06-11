pub mod std;
pub mod token;

use crate::session::source::FileId;
use token::Token;

pub trait Lexer {
    /// Method for tokenize file and return vector of tokens
    /// In implements recommended transmit Session for using source code which you can take with file_id. See example in crate::lexer/std.rs
    fn tokenize(&mut self, file_id: FileId) -> Vec<Token>;
}
