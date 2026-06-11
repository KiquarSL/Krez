use super::{
    Parser,
    ast::Stmt,
    expr::{ArithOp, CompOp, Expr, LogicOp, UnaryOp},
    types::Type,
};
use crate::lexer::token::{Keyword, TKind, Token};
use crate::report::{Level, Phase};
use crate::session::{Session, source::FileId};
use crate::{diag, help, info, span};

macro_rules! unexpected {
    ($self:expr, $token:expr,$notes:expr, $helps:expr, $($msg:tt)*) => {
        $self.session.emit_error(diag!(
            Level::Error,
            span!($self.file_id, $token),
            Phase::Parsing,
            $notes,
            $helps,
            $($msg)*
        ));
    };
}

macro_rules! expected {
    ($self:expr, $token:expr, $notes:expr, $helps:expr, $($msg:tt)*) => {
        $self.session.emit_error(diag!(
            Level::Error,
            span!($self.file_id, $token),
            Phase::Parsing,
            $notes,
            $helps,
            $($msg)*
        ));
    };
}

pub struct StdParser<'a> {
    tokens: Vec<Token>,
    index: usize,
    session: &'a mut Session,
    file_id: FileId,
}

impl<'a> Parser for StdParser<'a> {
    fn parse(&mut self, tokens: Vec<Token>, file_id: FileId) -> Vec<Stmt> {
        self.file_id = file_id;
        self.tokens = tokens;
        self.index = 0;
        let mut ast = vec![];
        while self.valid_pos() {
            let stmt = match self.stmt() {
                Ok(s) => s,
                Err(_) => continue,
            };
            ast.push(stmt);
        }
        ast
    }
}

impl<'a> StdParser<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self {
            index: 0,
            session,
            tokens: vec![],
            file_id: 0,
        }
    }

    pub fn new_full(session: &'a mut Session, tokens: Vec<Token>, file_id: FileId) -> Self {
        Self {
            index: 0,
            session,
            tokens,
            file_id,
        }
    }

    fn peek(&self, offset: u8) -> Token {
        let index = self.index + offset as usize;
        self.tokens[index].clone()
    }

    fn advance(&mut self, offset: u8) {
        self.index += offset as usize;
    }

    fn valid_pos(&self) -> bool {
        self.peek(0).kind != TKind::Eof
    }

    fn check(&mut self, kind: TKind) -> bool {
        if self.peek(0).kind == kind {
            self.advance(1);
            return true;
        }
        false
    }
    fn multi_check(&mut self, kinds: &[TKind]) -> bool {
        kinds.contains(&self.peek(0).kind)
    }

    pub fn parse_type(&mut self) -> Type {
        let start = self.peek(0);
        match start.kind {
            TKind::Id(id) => {
                self.advance(1);
                Type::from_str(&id, info!(start))
            }
            TKind::LBracket => {
                self.advance(1);
                let ty = self.parse_type();
                if !self.check(TKind::RBracket) {
                    let current = self.peek(0);
                    expected!(
                        self,
                        current,
                        vec![],
                        vec![help!(
                            span!(self.file_id, current.line, current.offset, 0),
                            "]",
                            false,
                            "Add ']' here"
                        )],
                        "Expected ']', found {}",
                        current
                    );
                }
                let tyi = ty.info();
                Type::Array(Box::new(ty), info!(tyi.line, tyi.offset - 1, tyi.len + 2))
            }
            TKind::Ampersand => {
                self.advance(1);
                let ty = self.parse_type();
                let tyi = ty.info();
                Type::Ptr(Box::new(ty), info!(tyi.line, tyi.offset - 1, tyi.len + 1))
            }
            _ => Type::Unknown,
        }
    }
    pub fn parse_fn_args(&mut self) -> Vec<(String, Type)> {
        let mut args = Vec::new();
        while self.valid_pos() && self.peek(0).kind != TKind::RParen {
            let current = self.peek(0);
            let id = match current.kind {
                TKind::Id(id) => {
                    self.advance(1);
                    id
                }
                _ => {
                    self.skip_until(&[TKind::RParen, TKind::Comma]);
                    let end = self.peek(0);
                    let len = end.pos - current.pos;
                    self.check(TKind::Comma);
                    args.push(("INVALID_IDENT".to_string(), Type::Unknown));
                    expected!(
                        self,
                        current,
                        vec![],
                        vec![help!(
                            span!(self.file_id, current.line, current.offset, len),
                            "ident: type",
                            false,
                            "Use currect argument declare"
                        )],
                        "Expected identificator, found {}",
                        current
                    );
                    continue;
                }
            };
            if !self.check(TKind::Colon) {
                let current = self.peek(0);
                expected!(
                    self,
                    current,
                    vec![],
                    vec![help!(
                        span!(self.file_id, current.line, current.offset, 0),
                        ": ",
                        false,
                        "Add ':' here"
                    )],
                    "Expected ':', found {}",
                    current
                );
                self.skip_until(&[TKind::RParen, TKind::Comma]);
                if self.check(TKind::Comma) {
                    continue;
                } else {
                    break;
                }
            }
            let ty = if !self.multi_check(&[
                TKind::Ampersand,
                TKind::LBracket,
                TKind::Id("".to_string()),
            ]) {
                let current = self.peek(0);
                expected!(
                    self,
                    current,
                    vec![],
                    vec![help!(
                        span!(self.file_id, current.line, current.offset, 0),
                        "type",
                        false,
                        "Add argument type here"
                    )],
                    "Expected type (identificator)/pointer (&)/array ([type]), found {}",
                    current
                );
                Type::Unknown
            } else {
                self.parse_type()
            };
            args.push((id, ty));
            self.check(TKind::Comma);
        }
        args
    }

    fn skip_until(&mut self, untils: &[TKind]) {
        while self.valid_pos() && !untils.contains(&self.peek(0).kind) {
            self.advance(1);
        }
    }

    fn parse_body(&mut self) -> Vec<Stmt> {
        let mut body = vec![];
        while self.valid_pos() && self.peek(0).kind != TKind::RBrace {
            let stmt = match self.stmt() {
                Ok(s) => s,
                Err(_) => continue,
            };
            body.push(stmt);
        }
        body
    }
}

impl<'a> StdParser<'a> {
    pub fn expr(&mut self) -> Result<Expr, ()> {
        Ok(self.logical()?)
    }
    fn logical(&mut self) -> Result<Expr, ()> {
        let mut left = self.comparison()?;
        while self.valid_pos() {
            let op_token = self.peek(0);
            let op = match op_token.kind {
                TKind::And => LogicOp::And,
                TKind::Or => LogicOp::Or,
                _ => break,
            };
            self.advance(1);
            let right = self.comparison()?;
            left = Expr::Logic(Box::new(left), op, Box::new(right), info!(op_token));
        }
        Ok(left)
    }
    fn comparison(&mut self) -> Result<Expr, ()> {
        let mut left = self.additive()?;
        while self.valid_pos() {
            let op_token = self.peek(0);
            let op = match op_token.kind {
                TKind::Gt => CompOp::Gt,
                TKind::Ge => CompOp::Ge,
                TKind::Lt => CompOp::Lt,
                TKind::Le => CompOp::Le,
                TKind::Eq => CompOp::Eq,
                TKind::Ne => CompOp::Ne,
                _ => break,
            };
            self.advance(1);
            let right = self.additive()?;
            left = Expr::Comp(Box::new(left), op, Box::new(right), info!(op_token));
        }
        Ok(left)
    }
    fn additive(&mut self) -> Result<Expr, ()> {
        let mut left = self.multiplicative()?;
        while self.valid_pos() {
            let op_token = self.peek(0);
            let op = match op_token.kind {
                TKind::Plus => ArithOp::Add,
                TKind::Minus => ArithOp::Sub,
                _ => break,
            };
            self.advance(1);
            let right = self.multiplicative()?;
            left = Expr::Arith(Box::new(left), op, Box::new(right), info!(op_token));
        }
        Ok(left)
    }
    fn multiplicative(&mut self) -> Result<Expr, ()> {
        let mut left = self.unary()?;
        while self.valid_pos() {
            let op_token = self.peek(0);
            let op = match op_token.kind {
                TKind::Star => ArithOp::Mul,
                TKind::Slash => ArithOp::Div,
                _ => break,
            };
            self.advance(1);
            let right = self.unary()?;
            left = Expr::Arith(Box::new(left), op, Box::new(right), info!(op_token));
        }
        Ok(left)
    }
    fn unary(&mut self) -> Result<Expr, ()> {
        let token = self.peek(0);
        Ok(match token.kind {
            TKind::Bang => {
                self.advance(1);
                let expr = self.primary()?;
                Expr::Unary(UnaryOp::Not, Box::new(expr), info!(token))
            }
            TKind::Minus => {
                self.advance(1);
                let expr = self.primary()?;
                Expr::Unary(UnaryOp::Neg, Box::new(expr), info!(token))
            }
            _ => self.primary()?,
        })
    }
    fn primary(&mut self) -> Result<Expr, ()> {
        let token = self.peek(0);
        match token.kind {
            TKind::Int(int) => {
                self.advance(1);
                Ok(Expr::Int(int, info!(token)))
            }
            TKind::Float(float) => {
                self.advance(1);
                Ok(Expr::Float(float, info!(token)))
            }
            TKind::Bool(boolean) => {
                self.advance(1);
                Ok(Expr::Bool(boolean, info!(token)))
            }
            TKind::Str(s) => {
                self.advance(1);
                Ok(Expr::Str(s, info!(token)))
            }
            TKind::Id(id) => {
                self.advance(1);
                Ok(Expr::Id(id, info!(token)))
            }
            _ => {
                self.session.emit_error(diag!(
                    Level::Error,
                    span!(self.file_id, token),
                    Phase::Parsing,
                    "Unexpected token {token}"
                ));
                self.advance(1);
                Err(())
            }
        }
    }
}

enum StmtKind {
    Fn,
    Declare,
    Assign,
    While,
    Expr,
}

fn define(pr: &StdParser) -> StmtKind {
    let token = pr.peek(0);
    match token.kind {
        TKind::Keyword(Keyword::Fix | Keyword::Mut) => StmtKind::Declare,
        TKind::Keyword(Keyword::Fn) => StmtKind::Fn,
        TKind::Keyword(Keyword::While) => StmtKind::While,
        TKind::Id(_) => StmtKind::Assign,
        _ => StmtKind::Expr,
    }
}

impl<'a> StdParser<'a> {
    pub fn stmt(&mut self) -> Result<Stmt, ()> {
        Ok(match define(self) {
            StmtKind::Expr => Stmt::Expr(self.expr()?),
            StmtKind::Fn => self.stmt_fn(),
            _ => todo!(),
        })
    }

    fn stmt_fn(&mut self) -> Stmt {
        self.advance(1);
        let id = self.peek(0);
        let id = match id.kind {
            TKind::Id(id) => {
                self.advance(1);
                id
            }
            _ => {
                unexpected!(
                    self,
                    id,
                    vec!["Add function identificator!"],
                    vec![],
                    "Expected identificator, found {id}"
                );
                "INVALID_IDENT".to_string()
            }
        };
        if !self.check(TKind::LParen) {
            let lparen = self.peek(0);
            expected!(
                self,
                lparen,
                vec![],
                vec![help!(
                    span!(self.file_id, lparen.line, lparen.offset, 0),
                    "(",
                    false,
                    "Add '(' here"
                )],
                "Expected '(', found {}",
                lparen
            );
        }
        let args = if self.check(TKind::RParen) {
            vec![]
        } else {
            let ags = self.parse_fn_args();
            if !self.check(TKind::RParen) {
                let rparen = self.peek(0);
                expected!(
                    self,
                    rparen,
                    vec![],
                    vec![help!(
                        span!(self.file_id, rparen.line, rparen.offset, 0),
                        ")",
                        false,
                        "Add ')' here"
                    )],
                    "Expected ')', found {}",
                    rparen
                );
            }
            ags
        };
        let mut ret_ty = None;
        if self.peek(0).kind != TKind::LBrace {
            ret_ty = Some(self.parse_type());
        }
        if !self.check(TKind::LBrace) {
            let lbrace = self.peek(0);
            expected!(
                self,
                lbrace,
                vec![],
                vec![help!(
                    span!(self.file_id, lbrace.line, lbrace.offset, 0),
                    "{",
                    false,
                    "Add '{{' here"
                )],
                "Expected '{{', found {}",
                lbrace
            );
        }
        let body = self.parse_body();
        if !self.check(TKind::RBrace) {
            let rbrace = self.peek(0);
            expected!(
                self,
                rbrace,
                vec![],
                vec![help!(
                    span!(self.file_id, rbrace.line, rbrace.offset, 0),
                    "}}",
                    false,
                    "Add '}}' here"
                )],
                "Expected '}}', found {}",
                rbrace
            );
        }
        Stmt::Func(id, args, ret_ty, body)
    }
}
