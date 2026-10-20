use crate::{BinOp, CompoundOp, Expr, ExprKind, Lit, UnaryOp, parser::Parser};
use TokenKind::*;
use roze_lexer::TokenKind;
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
    pub fn token(token: TokenKind) -> Option<Self> {
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
    pub(super) fn parse_lhs_expr(&mut self) -> Expr {
        let token = self.token();
        self.advance(); // skipped in every case

        let expr_kind = match token.kind {
            unary @ (Minus | Exclamation) => {
                let op = match unary {
                    Minus => UnaryOp::Minus,
                    Exclamation => UnaryOp::BitNot,
                    _ => unreachable!("`unary` doesn't guard all cases"),
                };

                let rhs = self.parse_expr_in(Prec::Unary).into();
                ExprKind::Unary(op, rhs)
            }
            Str => ExprKind::Lit(self.parse_str_lit(&token)),
            Num => ExprKind::Lit(self.parse_num_lit(&token)),
            val @ (True | False) => {
                let bool = Lit::Bool(val == True);
                ExprKind::Lit(bool)
            }
            Ident => {
                let sym = Symbol::intern(&self.s[&token]);
                ExprKind::Ident(sym)
            }
            k => panic!("unsupported token {k} while parsing lhs expression"),
        };

        Expr {
            kind: expr_kind,
            span: self.record_span(token.start),
        }
    }

    pub(super) fn parse_expr_in(&mut self, min_prec: Prec) -> Expr {
        let start_pos = self.token().start;
        let mut lhs = self.parse_lhs_expr();

        while let token = self.token()
            && let Some(this_prec) = Prec::token(token.kind)
            && this_prec > min_prec
        {
            self.advance(); // skipping the operator

            match token.kind {
                k @ (Plus | Minus | Asterisk | Slash | Percent | AmperAmper | PipePipe | Lt
                | LtEq | Gt | GtEq | EqEq) => {
                    let bin_op = match k {
                        Plus => BinOp::Add,
                        Minus => BinOp::Sub,
                        Asterisk => BinOp::Mul,
                        Slash => BinOp::Div,
                        Percent => BinOp::Mod,
                        AmperAmper => BinOp::And,
                        PipePipe => BinOp::Or,
                        Lt => BinOp::Lt,
                        LtEq => BinOp::LtEq,
                        Gt => BinOp::Gt,
                        GtEq => BinOp::GtEq,
                        _ => unreachable!("`bin_op` doesn't goard all cases"),
                    };

                    let rhs = self.parse_expr_in(this_prec);

                    lhs = Expr {
                        kind: ExprKind::Binary(lhs.into(), bin_op, rhs.into()),
                        span: self.record_span(start_pos),
                    }
                }
                k @ (PlusEq | MinusEq | AsteriskEq | SlashEq | PercentEq) => {
                    let compound_op = match k {
                        PlusEq => CompoundOp::Add,
                        MinusEq => CompoundOp::Sub,
                        AsteriskEq => CompoundOp::Mul,
                        SlashEq => CompoundOp::Div,
                        PercentEq => CompoundOp::Mod,
                        _ => unreachable!("`compound_op` doesn't guard all cases"),
                    };

                    let rhs = self.parse_expr_in(this_prec);

                    lhs = Expr {
                        kind: ExprKind::Compound(lhs.into(), compound_op, rhs.into()),
                        span: self.record_span(start_pos),
                    };
                }
                k => panic!("unsupported token {k} while parsing expression"),
            }
        }

        lhs
    }

    #[inline]
    pub(super) fn parse_expr(&mut self) -> Expr {
        self.parse_expr_in(Prec::Lowest)
    }
}
