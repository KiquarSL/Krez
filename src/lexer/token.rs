use std::mem::discriminant;
use strum;

/// Short name of TokenKind
pub type TKind = TokenKind;

/// Type of token with value
#[derive(Debug, Clone, strum::Display)]
pub enum TokenKind {
    /// +
    #[strum(to_string = "+")]
    Plus,
    /// -
    #[strum(to_string = "-")]
    Minus,
    /// *
    #[strum(to_string = "*")]
    Star,
    /// /
    #[strum(to_string = "/")]
    Slash,
    /// !
    #[strum(to_string = "!")]
    Bang,
    /// ,
    #[strum(to_string = ",")]
    Comma,
    /// .
    #[strum(to_string = ".")]
    Dot,
    /// ;
    #[strum(to_string = ";")]
    Semicolon,
    /// :
    #[strum(to_string = ":")]
    Colon,
    /// ::
    #[strum(to_string = "::")]
    Path,
    /// &
    #[strum(to_string = "&")]
    Ampersand,

    /// =
    #[strum(to_string = "=")]
    Assign,
    /// +=
    #[strum(to_string = "+=")]
    AssignPlus,
    /// -=
    #[strum(to_string = "-=")]
    AssignMinus,
    /// *=
    #[strum(to_string = "*=")]
    AssignStar,
    /// /=
    #[strum(to_string = "/=")]
    AssignSlash,

    /// ==
    #[strum(to_string = "==")]
    Eq,
    /// !=
    #[strum(to_string = "!=")]
    Ne,
    /// >
    #[strum(to_string = ">")]
    Gt,
    /// >=
    #[strum(to_string = ">=")]
    Ge,
    /// <
    #[strum(to_string = "<")]
    Lt,
    /// <=
    #[strum(to_string = "<=")]
    Le,

    /// (
    #[strum(to_string = "(")]
    LParen,
    /// )
    #[strum(to_string = ")")]
    RParen,
    /// [
    #[strum(to_string = "[")]
    LBracket,
    /// ]
    #[strum(to_string = "]")]
    RBracket,
    /// {
    #[strum(to_string = "{{")]
    LBrace,
    /// }
    #[strum(to_string = "}}")]
    RBrace,

    /// &&
    #[strum(to_string = "&&")]
    And,
    /// ||
    #[strum(to_string = "||")]
    Or,

    /// Keyword. Using Keyword enum as value
    #[strum(to_string = "{0}")]
    Keyword(Keyword),
    /// Identificator. String as value
    #[strum(to_string = "{0}")]
    Id(String),
    /// Boolean: true, false
    #[strum(to_string = "{0}")]
    Bool(bool),
    /// String literal
    #[strum(to_string = "\"{0}\"")]
    Str(String),
    /// Float value. Example: 3.14
    #[strum(to_string = "{0}")]
    Float(f64),
    /// Integer value. Example: 10
    #[strum(to_string = "{0}")]
    Int(i64),

    /// End Of File
    Eof,
}

impl PartialEq for TokenKind {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

/// Enum for typing in TokenType
#[derive(Debug, Clone, strum::Display, strum::EnumString)]
pub enum Keyword {
    /// fn
    #[strum(to_string = "fn")]
    Fn,
    /// while
    #[strum(to_string = "while")]
    While,
    /// fix (Full: Fixed)
    #[strum(to_string = "fix")]
    Fix,
    /// mut (Full: Muttable)
    #[strum(to_string = "mut")]
    Mut,
    /// break
    #[strum(to_string = "break")]
    Break,
    /// continue
    #[strum(to_string = "continue")]
    Continue,
    /// if
    #[strum(to_string = "if")]
    If,
    /// elif
    #[strum(to_string = "elif")]
    Elif,
    /// else
    #[strum(to_string = "else")]
    Else,
    /// ret (Full: Return)
    #[strum(to_string = "ret")]
    Ret,
    /// extern
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
    /// Type of token with value
    pub kind: TokenKind,
    /// Line number where located this token
    pub line: usize,
    /// Offset from line start
    pub offset: usize,
    /// Absolute position in source text
    pub pos: usize,
    /// Length
    pub len: usize,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Token {
    /** Return new Token
    # Examples

    ```
    // If source: fix a = 4;
    let fix = Token::new(TKind::Keyword(Keyword::Fix), 0, 0, 0, 3);
    let ident = Token::new(TKind::id("a"), 0, 4, 4, 1);
    let assign = Token::new(TKind::Assign, 0, 6, 6, 1);
    ```
    */
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
