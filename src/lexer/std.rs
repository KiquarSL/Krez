use super::{
    Lexer,
    token::{Keyword, TKind, Token},
};
use crate::report::{Level, Phase};
use crate::session::{
    Session,
    source::{FileId, Source},
};
use crate::{diag, help, span};
use std::str::FromStr;

macro_rules! double_token {
    ($self:expr, $second:expr, $then:ident, $else:ident) => {
        if $self.peek(1) == Some($second) {
            $self.push(TKind::$then, $self.line, $self.offset, $self.pos, 2);
            $self.advance(2);
        } else {
            $self.push(TKind::$else, $self.line, $self.offset, $self.pos, 1);
            $self.advance(1);
        }
    };
}

macro_rules! only_double_token {
    ($self:expr, $full:expr, $second:expr, $kind:ident, $current:expr) => {
        if $self.peek(1) == Some($second) {
            $self.push(TKind::$kind, $self.line, $self.offset, $self.pos, 2);
            $self.advance(2);
        } else {
            let h = help!(
                span!($self.file_id, $self.line, $self.offset, 1),
                $full,
                false,
                "Meybe you mean `{}`?",
                $full
            );
            $self.session.emit_error(diag!(
                Level::Error,
                span!($self.file_id, $self.line, $self.offset, 1),
                Phase::Lexing,
                vec![],
                vec![h],
                "Unknown character '{}'",
                $current
            ));
            $self.advance(1);
        }
    };
}

pub struct StdLexer<'a> {
    pos: usize,
    file_id: FileId,
    tokens: Vec<Token>,
    session: &'a mut Session,
    chars: Vec<char>,

    line: usize,
    offset: usize,
}

impl<'a> Lexer for StdLexer<'a> {
    fn tokenize(&mut self, file_id: FileId) -> Vec<Token> {
        self.file_id = file_id;
        self.chars = self.src().text.chars().collect();
        while self.valid_pos() {
            let current = self.peek(0);
            let current = match current {
                Some(ch) => ch,
                None => break,
            };
            let line = self.line;
            let offset = self.offset;
            let pos = self.pos;

            match current {
                ch if ch.is_whitespace() => self.advance(1),
                ch if "(){}[],".contains(ch) => {
                    self.push_one(match ch {
                        '(' => TKind::LParen,
                        ')' => TKind::RParen,
                        '[' => TKind::LBracket,
                        ']' => TKind::RBracket,
                        '{' => TKind::LBrace,
                        '}' => TKind::RBrace,
                        ',' => TKind::Comma,
                        _ => unreachable!(),
                    });
                    self.advance(1);
                }
                '+' => double_token!(self, '=', AssignPlus, Plus),
                '-' => double_token!(self, '=', AssignMinus, Minus),
                '*' => double_token!(self, '=', AssignStar, Star),

                '=' => double_token!(self, '=', Eq, Assign),
                '!' => double_token!(self, '=', Ne, Bang),
                '>' => double_token!(self, '=', Ge, Gt),
                '<' => double_token!(self, '=', Le, Lt),

                '&' => double_token!(self, '&', And, Ampersand),
                ':' => double_token!(self, ':', Path, Colon),
                '|' => only_double_token!(self, "||", '|', Or, current),

                '/' => {
                    if self.peek(1) == Some('=') {
                        self.push(TKind::AssignSlash, line, offset, pos, 2);
                        self.advance(2);
                    } else if self.peek(1) == Some('/') {
                        // short comment
                        self.advance(2);
                        while self.valid_pos() && self.peek(0) != Some('\n') {
                            self.advance(1);
                        }
                        self.advance(1);
                    } else if self.peek(1) == Some('*') {
                        // long comment
                        self.advance(2);
                        while self.valid_pos()
                            && self.peek(0) != Some('*')
                            && self.peek(0) != Some('/')
                        {
                            self.advance(1);
                        }
                        self.advance(2);
                    } else {
                        self.push(TKind::Slash, line, offset, pos, 1);
                        self.advance(1);
                    }
                }
                ch if ch.is_digit(10) => {
                    let mut buffer = String::new();
                    while self.valid_pos() {
                        let current = self.peek(0);
                        self.advance(1);
                        if let Some(ch) = current {
                            if ch == '.' || ch.is_digit(10) {
                                buffer.push(ch);
                            } else {
                                break;
                            }
                        }
                    }
                    let kind = if buffer.contains('.') {
                        let f = match buffer.parse::<f64>() {
                            Ok(f) => f,
                            Err(err) => {
                                self.session.emit_error(diag!(
                                    Level::Error,
                                    span!(file_id, line, offset, buffer.len()),
                                    Phase::Lexing,
                                    "Failed parse number: {err}"
                                ));
                                0.0
                            }
                        };
                        TKind::Float(f)
                    } else {
                        let i = match buffer.parse::<i64>() {
                            Ok(f) => f,
                            Err(err) => {
                                self.session.emit_error(diag!(
                                    Level::Error,
                                    span!(file_id, line, offset, buffer.len()),
                                    Phase::Lexing,
                                    "Failed parse number: {err}"
                                ));
                                0
                            }
                        };
                        TKind::Int(i)
                    };
                    self.push(kind, line, offset, pos, buffer.len());
                }
                ch if ch.is_alphabetic() => {
                    let mut buffer = String::new();
                    while self.valid_pos() {
                        let current = self.peek(0);
                        if let Some(ch) = current {
                            if ch.is_alphabetic() || ch.is_digit(10) || ch == '_' {
                                buffer.push(ch);
                                self.advance(1);
                            } else {
                                break;
                            }
                        }
                    }
                    let len = buffer.len();
                    self.push(
                        match buffer.as_str() {
                            "true" => TKind::Bool(true),
                            "false" => TKind::Bool(false),
                            b if let Ok(kw) = Keyword::from_str(&b) => TKind::Keyword(kw),
                            _ => TKind::Id(buffer),
                        },
                        line,
                        offset,
                        pos,
                        len,
                    );
                }
                '"' => {
                    let mut buffer = String::new();
                    self.advance(1);
                    while self.valid_pos() {
                        let current = self.peek(0);
                        self.advance(1);
                        if let Some(ch) = current {
                            if ch != '"' {
                                buffer.push(ch);
                            } else {
                                break;
                            }
                        }
                    }
                    self.advance(1);
                    let len = buffer.len() + 2;
                    self.push(TKind::Str(buffer), line, offset, pos, len);
                }
                _ => {
                    self.session.emit_error(diag!(
                        Level::Error,
                        span!(file_id, self.line, self.offset, 1),
                        Phase::Lexing,
                        "Unknown character '{current}'"
                    ));
                    self.advance(1);
                }
            }
        }
        self.push(TKind::Eof, self.line, self.offset, self.pos, 1);
        self.tokens.clone()
    }
}

impl<'a> StdLexer<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self {
            file_id: 0,
            pos: 0,
            session,
            tokens: vec![],
            chars: vec![],
            line: 0,
            offset: 0,
        }
    }

    fn src(&self) -> &Source {
        &self.session.sources()[self.file_id]
    }

    fn peek(&self, offset: u8) -> Option<char> {
        let index = self.pos + offset as usize;
        self.chars.get(index).copied()
    }

    fn advance(&mut self, offset: usize) {
        for _ in 0..offset {
            if self.peek(0) == Some('\n') {
                self.offset = 0;
                self.line += 1;
            } else {
                self.offset += 1;
            }
            self.pos += 1;
        }
    }

    fn push(&mut self, kind: TKind, line: usize, offset: usize, pos: usize, len: usize) {
        self.tokens.push(Token::new(kind, line, offset, pos, len));
    }

    fn push_one(&mut self, kind: TKind) {
        self.tokens
            .push(Token::new(kind, self.line, self.offset, self.pos, 1));
    }

    fn valid_pos(&self) -> bool {
        self.pos < self.src().len
    }
}
