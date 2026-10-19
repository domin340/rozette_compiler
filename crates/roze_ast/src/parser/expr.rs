use crate::{
    Expr, ExprKind, Span, UnaryOp, ast,
    parser::{Parser, span},
};
use TokenKind::*;
use roze_lexer::{Token, TokenKind};

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

impl Parser {
    pub(super) fn parse_lhs_expr(&mut self) -> Expr {
        let Token { kind, start, .. } = self.token();
        let expr_kind = match kind {
            unary @ (Minus | Exclamation) => {
                let op = match unary {
                    Minus => UnaryOp::Minus,
                    Exclamation => UnaryOp::BitNot,
                    _ => panic!("`unary` does not guard all the possible cases"),
                };

                self.advance(); // advance unary operator

                let rhs = self.parse_expr_in(Prec::Unary).into();
                ExprKind::Unary(op, rhs)
            }
            kind => panic!("Unexpected: {kind} found in lhs of expr."),
        };

        ast!(expr, expr_kind, self.record_span(start))
    }

    pub(super) fn parse_expr_in(&mut self, prec: Prec) -> Expr {
        todo!()
    }

    #[inline]
    pub(super) fn parse_expr(&mut self) -> Expr {
        self.parse_expr_in(Prec::Lowest)
    }
}
