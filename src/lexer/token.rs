use std::mem::discriminant;
use strum;

pub type TKind = TokenKind;

#[derive(Debug, Clone, strum::Display)]
pub enum TokenKind {
    #[strum(to_string = "+")]
    Plus,
    #[strum(to_string = "-")]
    Minus,
    #[strum(to_string = "*")]
    Star,
    #[strum(to_string = "/")]
    Slash,
    #[strum(to_string = "!")]
    Bang,

    #[strum(to_string = "=")]
    Assign,
    #[strum(to_string = "+=")]
    AssignPlus,
    #[strum(to_string = "-=")]
    AssignMinus,
    #[strum(to_string = "*=")]
    AssignStar,
    #[strum(to_string = "/=")]
    AssignSlash,

    #[strum(to_string = "==")]
    Eq,
    #[strum(to_string = "!=")]
    Ne,
    #[strum(to_string = ">")]
    Gt,
    #[strum(to_string = ">=")]
    Ge,
    #[strum(to_string = "<")]
    Lt,
    #[strum(to_string = "<=")]
    Le,

    #[strum(to_string = "(")]
    LParen,
    #[strum(to_string = ")")]
    RParen,
    #[strum(to_string = "[")]
    LBracket,
    #[strum(to_string = "]")]
    RBracket,
    #[strum(to_string = "{{")]
    LBrace,
    #[strum(to_string = "}}")]
    RBrace,

    #[strum(to_string = "&&")]
    And,
    #[strum(to_string = "||")]
    Or,

    #[strum(to_string = "{0}")]
    Keyword(Keyword),
    #[strum(to_string = "{0}")]
    Id(String),
    #[strum(to_string = "{0}")]
    Bool(bool),
    #[strum(to_string = "\"{0}\"")]
    Str(String),
    #[strum(to_string = "{0}")]
    Float(f64),
    #[strum(to_string = "{0}")]
    Int(i64),

    Eof,
}

impl PartialEq for TokenKind {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

#[derive(Debug, Clone, strum::Display, strum::EnumString)]
pub enum Keyword {
    #[strum(to_string = "fn")]
    Fn,
    #[strum(to_string = "while")]
    While,
    #[strum(to_string = "fix")]
    Fix,
    #[strum(to_string = "mut")]
    Mut,
    #[strum(to_string = "break")]
    Break,
    #[strum(to_string = "continue")]
    Continue,
    #[strum(to_string = "if")]
    If,
    #[strum(to_string = "elif")]
    Elif,
    #[strum(to_string = "else")]
    Else,
    #[strum(to_string = "ret")]
    Ret,
    #[strum(to_string = "extern")]
    Extern,
}

impl PartialEq for Keyword {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub offset: usize,
    pub pos: usize,
    pub len: usize,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, offset: usize, pos: usize, len: usize) -> Self {
        Self {
            kind,
            len,
            line,
            offset,
            pos,
        }
    }
}
