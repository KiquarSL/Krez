use super::{
    Parser,
    ast::Stmt,
    expr::{ArithOp, CompOp, Expr, LogicOp, UnaryOp},
};
use crate::lexer::token::{TKind, Token};
use crate::report::{Level, Phase};
use crate::session::{Session, source::FileId};
use crate::{diag, info, span};

pub struct StdParser<'a> {
    tokens: Vec<Token>,
    index: usize,
    session: &'a mut Session,
    file_id: FileId,
}

impl<'a> Parser for StdParser<'a> {
    fn parse(&mut self, tokens: Vec<Token>, file_id: FileId) -> Vec<Stmt> {
        self.file_id = file_id;
        let ast = vec![];
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
}

impl<'a> StdParser<'a> {
    pub fn expr(&mut self) -> Result<Expr, ()> {
        Ok(self.logical()?)
    }
    fn logical(&mut self) -> Result<Expr, ()> {
        let mut left = self.comparison()?;
        loop {
            let op_token = self.peek(0);
            let op = match op_token.kind {
                TKind::And => LogicOp::And,
                TKind::Or => LogicOp::Or,
                _ => break,
            };
            self.advance(1);
            let right = self.additive()?;
            left = Expr::Logic(Box::new(left), op, Box::new(right), info!(op_token));
        }
        Ok(left)
    }
    fn comparison(&mut self) -> Result<Expr, ()> {
        let mut left = self.additive()?;
        loop {
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
        loop {
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
        loop {
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
