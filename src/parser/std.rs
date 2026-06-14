use super::{
    Parser,
    ast::{AssignOp, MutKind, Stmt, UseItem},
    expr::{ArithOp, CompOp, Expr, LogicOp, UnaryOp},
    types::Type,
};
use crate::lexer::token::{Keyword, TKind, Token};
use crate::report::{Level, Phase};
use crate::session::{Session, source::FileId};
use crate::{diag, help, info, span};

macro_rules! emit_error {
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

    fn back_peek(&self, offset: u8) -> Token {
        let index = self.index - offset as usize;
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
                    emit_error!(
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
                    emit_error!(
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
                emit_error!(
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
                emit_error!(
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

    pub fn parse_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        while self.valid_pos() && self.peek(0).kind != TKind::RParen {
            let expr = match self.expr() {
                Ok(expr) => expr,
                Err(_) => Expr::Invalid,
            };
            args.push(expr);
            self.check(TKind::Comma);
        }
        args
    }

    /// Parse path: a::b::c
    fn parse_path(&mut self) -> Vec<UseItem> {
        let mut path = vec![];
        while self.valid_pos() {
            let token = self.peek(0);
            match token.kind {
                TKind::Id(id) => {
                    self.advance(1);
                    path.push(UseItem::Path(id));
                }
                TKind::Path => {
                    self.advance(1);
                    continue;
                }
                _ => break,
            }
        }
        path
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
                let mut path = vec![id];
                self.advance(1);
                while self.valid_pos() {
                    if !self.check(TKind::Path) {
                        break;
                    }
                    if let TKind::Id(id) = self.peek(0).kind {
                        path.push(id);
                        self.advance(1);
                    }
                }
                let end = self.peek(0);
                if self.check(TKind::LParen) {
                    let args = self.parse_args();
                    if !self.check(TKind::RParen) {
                        let rparen = self.back_peek(1);
                        emit_error!(
                            self,
                            rparen,
                            vec![],
                            vec![help!(
                                span!(self.file_id, rparen.line, rparen.offset + rparen.len, 0),
                                ")",
                                false,
                                "Add ')' here"
                            )],
                            "Expected ')', found {}",
                            rparen
                        );
                    }
                    let id = Expr::Id(path, info!(token.line, token.offset, end.pos - token.pos));
                    Ok(Expr::Call(Box::new(id.clone()), args, id.info()))
                } else {
                    Ok(Expr::Id(
                        path,
                        info!(token.line, token.offset, end.pos - token.pos),
                    ))
                }
            }
            TKind::LParen => {
                self.advance(1);
                let expr = self.expr()?;
                if !self.check(TKind::RParen) {
                    let rparen = self.back_peek(1);
                    emit_error!(
                        self,
                        rparen,
                        vec![],
                        vec![help!(
                            span!(self.file_id, rparen.line, rparen.offset + rparen.len, 0),
                            ")",
                            false,
                            "Add ')' here"
                        )],
                        "Expected ')', found {}",
                        rparen
                    );
                }
                Ok(expr)
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
    IfElse,
    Declare,
    Assign,
    While,
    Return,
    Break,
    Continue,
    Expr,
    Use,
}

fn define(pr: &StdParser) -> StmtKind {
    let token = pr.peek(0);
    let next_token = pr.peek(1);

    match (token.kind, next_token.kind) {
        (TKind::Keyword(Keyword::Fix | Keyword::Mut), _) => StmtKind::Declare,
        (TKind::Keyword(Keyword::If), _) => StmtKind::IfElse,
        (TKind::Keyword(Keyword::Ret), _) => StmtKind::Return,
        (TKind::Keyword(Keyword::Fn), _)
        | (TKind::Keyword(Keyword::Pub), TKind::Keyword(Keyword::Fn)) => StmtKind::Fn,
        (TKind::Keyword(Keyword::While), _) => StmtKind::While,
        (
            TKind::Id(_),
            TKind::Assign
            | TKind::AssignPlus
            | TKind::AssignMinus
            | TKind::AssignSlash
            | TKind::AssignStar,
        ) => StmtKind::Assign,
        (TKind::Star, TKind::Id(_)) => StmtKind::Assign,
        (TKind::Keyword(Keyword::Break), _) => StmtKind::Break,
        (TKind::Keyword(Keyword::Continue), _) => StmtKind::Continue,
        (TKind::Keyword(Keyword::Use), _)
        | (TKind::Keyword(Keyword::Pub), TKind::Keyword(Keyword::Use)) => StmtKind::Use,
        _ => StmtKind::Expr,
    }
}

impl<'a> StdParser<'a> {
    pub fn stmt(&mut self) -> Result<Stmt, ()> {
        Ok(match define(self) {
            StmtKind::Expr => Stmt::Expr(self.expr()?),
            StmtKind::Declare => self.stmt_declare(),
            StmtKind::Assign => self.stmt_assign(),
            StmtKind::Fn => self.stmt_fn(),
            StmtKind::IfElse => self.stmt_if_else(),
            StmtKind::Return => self.stmt_return(),
            StmtKind::Break | StmtKind::Continue => self.stmt_break_continue(),
            StmtKind::While => self.stmt_while_loop(),
            StmtKind::Use => self.stmt_use(),
        })
    }

    fn stmt_break_continue(&mut self) -> Stmt {
        let stmt = match self.peek(0).kind {
            TKind::Keyword(kw) => {
                if kw == Keyword::Break {
                    Stmt::Break
                } else {
                    Stmt::Continue
                }
            }
            _ => unreachable!(),
        };
        self.advance(1);
        if !self.check(TKind::Semicolon) {
            let semicolon = self.peek(0);
            let new = self.back_peek(1);
            emit_error!(
                self,
                semicolon,
                vec![],
                vec![help!(
                    span!(self.file_id, new.line, new.offset + new.len, 0),
                    ";",
                    false,
                    "Add ';' here"
                )],
                "Expected ';', found {}",
                semicolon
            );
        }
        stmt
    }

    fn stmt_use(&mut self) -> Stmt {
        let is_pub = if let TKind::Keyword(Keyword::Pub) = self.peek(0).kind {
            self.advance(1);
            true
        } else {
            false
        };
        self.advance(1);
        let path = vec![self.parse_path()];
        if !self.check(TKind::Semicolon) {
            let semicolon = self.peek(0);
            let new = self.back_peek(1);
            emit_error!(
                self,
                semicolon,
                vec![],
                vec![help!(
                    span!(self.file_id, new.line, new.offset + new.len, 0),
                    ";",
                    false,
                    "Add ';' here"
                )],
                "Expected ';', found {}",
                semicolon
            );
        }
        Stmt::Use(is_pub, path)
    }

    fn stmt_while_loop(&mut self) -> Stmt {
        self.advance(1);
        let cond = match self.expr() {
            Ok(val) => val,
            Err(_) => Expr::Invalid,
        };
        if !self.check(TKind::LBrace) {
            let lbrace = self.peek(0);
            emit_error!(
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
            return Stmt::While(cond, vec![]);
        }
        let body = self.parse_body();
        if !self.check(TKind::RBrace) {
            let rbrace = self.peek(0);
            emit_error!(
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
        return Stmt::While(cond, body);
    }

    fn stmt_return(&mut self) -> Stmt {
        self.advance(1);
        let val = if self.check(TKind::Semicolon) {
            None
        } else {
            let val = match self.expr() {
                Ok(val) => val,
                Err(_) => Expr::Invalid,
            };
            if !self.check(TKind::Semicolon) {
                let semicolon = self.peek(0);
                let new = self.back_peek(1);
                emit_error!(
                    self,
                    semicolon,
                    vec![],
                    vec![help!(
                        span!(self.file_id, new.line, new.offset + new.len, 0),
                        ";",
                        false,
                        "Add ';' here"
                    )],
                    "Expected ';', found {}",
                    semicolon
                );
            }
            Some(val)
        };
        Stmt::Return(val)
    }

    fn stmt_if_else(&mut self) -> Stmt {
        self.advance(1);
        let mut has_else = false;
        let mut branches = vec![];
        while self.valid_pos() {
            let cond = if has_else {
                None
            } else {
                match self.expr() {
                    Ok(expr) => Some(expr),
                    Err(_) => Some(Expr::Invalid),
                }
            };

            if !self.check(TKind::LBrace) {
                let lbrace = self.peek(0);
                emit_error!(
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
                break;
            }
            let body = self.parse_body();
            if !self.check(TKind::RBrace) {
                let rbrace = self.peek(0);
                emit_error!(
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
            branches.push((cond, body));
            if let TKind::Keyword(kw) = self.peek(0).kind {
                if kw == Keyword::Elif {
                    self.advance(1);
                    continue;
                } else if kw == Keyword::Else {
                    self.advance(1);
                    has_else = true;
                    continue;
                }
            }

            break;
        }
        Stmt::IfElse(branches)
    }

    fn stmt_assign(&mut self) -> Stmt {
        let deref = if self.peek(0).kind == TKind::Star {
            self.advance(1);
            true
        } else {
            false
        };
        let id = self.peek(0);
        let id = match id.kind {
            TKind::Id(id) => {
                self.advance(1);
                id
            }
            _ => {
                emit_error!(
                    self,
                    id,
                    vec![],
                    vec![help!(
                        span!(self.file_id, id.line, id.offset, 0),
                        "var_ident",
                        false,
                        "Add variable identificator here"
                    )],
                    "Expected identificator, found {id}"
                );
                self.skip_until(&[TKind::Semicolon]);
                self.advance(1);
                return Stmt::Assign(
                    deref,
                    String::from("INVALID_IDENT"),
                    AssignOp::default(),
                    Expr::Invalid,
                );
            }
        };
        let assign = self.peek(0);
        let assign = match assign.kind {
            TKind::Assign => AssignOp::default(),
            TKind::AssignPlus => AssignOp::Plus,
            TKind::AssignMinus => AssignOp::Minus,
            TKind::AssignStar => AssignOp::Star,
            TKind::AssignSlash => AssignOp::Slash,
            _ => {
                emit_error!(
                    self,
                    assign,
                    vec![],
                    vec![help!(
                        span!(self.file_id, assign.line, assign.offset, 0),
                        "= ",
                        false,
                        "Add assign operator here"
                    )],
                    "Expected =/+=/-=/*=//=, found {id}"
                );
                self.skip_until(&[TKind::Semicolon]);
                self.advance(1);
                return Stmt::Assign(deref, id, AssignOp::default(), Expr::Invalid);
            }
        };
        self.advance(1);

        if self.peek(0).kind == TKind::Semicolon {
            let semicolon = self.peek(0);
            emit_error!(
                self,
                semicolon,
                vec![],
                vec![help!(
                    span!(self.file_id, semicolon.line, semicolon.offset, 0),
                    "expression",
                    false,
                    "Add expression here"
                )],
                "Expected expression, found {}",
                semicolon
            );
            self.advance(1);
            return Stmt::Assign(deref, id, assign, Expr::Invalid);
        }
        let val = match self.expr() {
            Ok(expr) => expr,
            Err(_) => {
                self.skip_until(&[TKind::Semicolon]);
                Expr::Invalid
            }
        };
        if !self.check(TKind::Semicolon) {
            let semicolon = self.peek(0);
            let new = self.back_peek(1);
            emit_error!(
                self,
                semicolon,
                vec![],
                vec![help!(
                    span!(self.file_id, new.line, new.offset + new.len, 0),
                    ";",
                    false,
                    "Add ';' here"
                )],
                "Expected ';', found {}",
                semicolon
            );
        }
        Stmt::Assign(deref, id, assign, val)
    }

    fn stmt_declare(&mut self) -> Stmt {
        let mut_kind = self.peek(0);
        let mut_kind = match mut_kind.kind {
            TKind::Keyword(Keyword::Mut) => MutKind::Mutable,
            TKind::Keyword(Keyword::Fix) => MutKind::Fixed,
            _ => unreachable!(),
        };
        self.advance(1);
        let id = self.peek(0);
        let id = match id.kind {
            TKind::Id(id) => {
                self.advance(1);
                id
            }
            _ => {
                emit_error!(
                    self,
                    id,
                    vec![],
                    vec![help!(
                        span!(self.file_id, id.line, id.offset, 0),
                        "var_ident",
                        false,
                        "Add variable identificator here"
                    )],
                    "Expected identificator, found {id}"
                );
                self.skip_until(&[TKind::Semicolon]);
                self.advance(1);
                return Stmt::Declare(
                    mut_kind,
                    String::from("INVALID_IDENT"),
                    Type::Unknown,
                    Expr::Invalid,
                );
            }
        };
        let ty = if self.check(TKind::Colon) {
            self.parse_type()
        } else {
            Type::Unknown
        };
        if !self.check(TKind::Assign) {
            let assign = self.peek(0);
            emit_error!(
                self,
                assign,
                vec![],
                vec![help!(
                    span!(self.file_id, assign.line, assign.offset, 0),
                    "= ",
                    false,
                    "Add '=' here"
                )],
                "Expected '=', found {}",
                assign
            );
            self.skip_until(&[TKind::Semicolon]);
            self.advance(1);
            return Stmt::Declare(mut_kind, id, ty, Expr::Invalid);
        }
        if self.peek(0).kind == TKind::Semicolon {
            let semicolon = self.peek(0);
            emit_error!(
                self,
                semicolon,
                vec![],
                vec![help!(
                    span!(self.file_id, semicolon.line, semicolon.offset, 0),
                    "expression",
                    false,
                    "Add expression here"
                )],
                "Expected expression, found {}",
                semicolon
            );
            self.advance(1);
            return Stmt::Declare(mut_kind, id, ty, Expr::Invalid);
        }
        let val = match self.expr() {
            Ok(expr) => expr,
            Err(_) => {
                self.skip_until(&[TKind::Semicolon]);
                Expr::Invalid
            }
        };
        if !self.check(TKind::Semicolon) {
            let semicolon = self.peek(0);
            let new = self.back_peek(1);
            emit_error!(
                self,
                semicolon,
                vec![],
                vec![help!(
                    span!(self.file_id, new.line, new.offset + new.len, 0),
                    ";",
                    false,
                    "Add ';' here"
                )],
                "Expected ';', found {}",
                semicolon
            );
        }
        Stmt::Declare(mut_kind, id, ty, val)
    }

    fn stmt_fn(&mut self) -> Stmt {
        let is_pub = if let TKind::Keyword(Keyword::Pub) = self.peek(0).kind {
            self.advance(1);
            true
        } else {
            false
        };
        self.advance(1);
        let id = self.peek(0);
        let id = match id.kind {
            TKind::Id(id) => {
                self.advance(1);
                id
            }
            _ => {
                emit_error!(
                    self,
                    id,
                    vec![],
                    vec![help!(
                        span!(self.file_id, id.line, id.offset, 0),
                        "func_name",
                        false,
                        "Add function identificator here"
                    )],
                    "Expected identificator, found {id}"
                );
                "INVALID_IDENT".to_string()
            }
        };
        if !self.check(TKind::LParen) {
            let lparen = self.peek(0);
            emit_error!(
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
                emit_error!(
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
            emit_error!(
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
            emit_error!(
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
        Stmt::Func(is_pub, id, args, ret_ty, body)
    }
}
