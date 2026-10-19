use crate::{Expr, ExprKind, Lit, UnaryOp, ast, parser::Parser};
use TokenKind::*;
use roze_lexer::{Token, TokenKind};
use roze_span::Symbol;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    Unary,
    Product,
    Sum,
    Comp,
    Eq,
    Logical,
    Assignment,
    #[default]
    Lowest,
}

impl Prec {
    pub fn op_prec(token: TokenKind) -> Option<Self> {
        Some(match token {
            Asterisk | Slash | Percent => Self::Product,
            Lt | Gt | LtEq | GtEq => Self::Comp,
            EqEq | NotEq => Self::Eq,
            Eq | AsteriskEq | SlashEq | PercentEq | PlusEq | MinusEq => Self::Assignment,
            AmperAmper | PipePipe => Self::Logical,
            _ => return None,
        })
    }
}

impl Parser<'_> {
    fn parse_str_lit(&mut self, t: &Token) -> Lit {
        let s = &self.s[t];

        let bytes = s.as_bytes();
        if bytes.len() < 2 // the length must be sufficient, at least have 2 quotes
        || !matches!(bytes[0], b'\'' | b'"') // first character must be quote
        // both are the same quotes
        || bytes[0] != bytes.last().copied().unwrap()
        {
            panic!("Expected {s} str lit to start and end with the same quote.");
        }

        // intern inner data and return lit
        let s = &s[1..(s.len() - 1)];
        let str_sym = Symbol::intern(s);

        Lit::Str(str_sym)
    }

    fn parse_num_lit(&mut self, t: &Token) -> Lit {
        todo!()
    }

    pub(super) fn parse_lhs_expr(&mut self) -> Expr {
        let token = self.token();
        let expr_kind = match token.kind {
            unary @ (Minus | Exclamation) => {
                self.advance(); // advance unary operator

                let op = match unary {
                    Minus => UnaryOp::Minus,
                    Exclamation => UnaryOp::BitNot,
                    _ => panic!("`unary` does not guard all the possible cases"),
                };

                let rhs = self.parse_expr_in(Prec::Unary).into();
                ExprKind::Unary(op, rhs)
            }
            Str => {
                self.advance();
                ExprKind::Lit(self.parse_str_lit(&token))
            }
            Num => {
                self.advance();
                ExprKind::Lit(self.parse_num_lit(&token))
            }
            bool @ (True | False) => {
                self.advance();
                let val = bool == True;
                ExprKind::Lit(Lit::Bool(val))
            }
            Ident => {
                self.advance();
                let sym = Symbol::intern(&self.s[&token]);
                ExprKind::Ident(sym)
            }
            kind => panic!("Unexpected: {kind} found in lhs of expr."),
        };

        ast!(expr, expr_kind, self.record_span(token.start))
    }

    pub(super) fn parse_expr_in(&mut self, prec: Prec) -> Expr {
        todo!()
    }

    #[inline]
    pub(super) fn parse_expr(&mut self) -> Expr {
        self.parse_expr_in(Prec::Lowest)
    }
}
