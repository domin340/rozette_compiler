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
            Plus | Minus => Self::Sum,
            Lt | Gt | LtEq | GtEq => Self::Comp,
            EqEq | NotEq => Self::Eq,
            Eq | AsteriskEq | SlashEq | PercentEq | PlusEq | MinusEq => Self::Assignment,
            AmperAmper | PipePipe => Self::Logical,
            _ => return None,
        })
    }
}

impl Parser<'_> {
    pub(crate) fn parse_lhs_expr(&mut self) -> Expr {
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

    pub(crate) fn parse_expr_in(&mut self, min_prec: Prec) -> Expr {
        let start_pos = self.token().start;
        let mut lhs = self.parse_lhs_expr();

        while let token = self.token()
            && let Some(this_prec) = Prec::token(token.kind)
            && this_prec <= min_prec
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
                Eq => {
                    let rhs = self.parse_expr_in(this_prec);
                    lhs = Expr {
                        kind: ExprKind::Assign(lhs.into(), rhs.into()),
                        span: self.record_span(start_pos),
                    }
                }
                k => panic!("unsupported token {k} while parsing expression"),
            }
        }

        lhs
    }

    #[inline]
    pub(crate) fn parse_expr(&mut self) -> Expr {
        self.parse_expr_in(Prec::Lowest)
    }
}

#[cfg(test)]
mod tests {
    use roze_span::{Span, Symbol};

    use crate::{BinOp, Expr, ExprKind, Lit, Parser, UnaryOp};

    #[test]
    fn parse_literals_and_ident() {
        let hello = Symbol::intern("hello");
        let decimal = Symbol::intern("12.3e+2");
        let hex = Symbol::intern("0x3faFA");
        let identifier = Symbol::intern("identifier");

        let mut parser = Parser::new(r#""hello" 12.3e+2 0x3faFA true false identifier"#);

        assert_eq!(
            Expr {
                kind: ExprKind::Lit(Lit::Str(hello)),
                span: Span::new(0, 7),
            },
            parser.parse_expr()
        );
        assert_eq!(
            Expr {
                kind: ExprKind::Lit(Lit::Num(decimal)),
                span: Span::new(8, 15),
            },
            parser.parse_expr()
        );
        assert_eq!(
            Expr {
                kind: ExprKind::Lit(Lit::Num(hex)),
                span: Span::new(16, 23),
            },
            parser.parse_expr()
        );
        assert_eq!(
            Expr {
                kind: ExprKind::Lit(Lit::Bool(true)),
                span: Span::new(24, 28),
            },
            parser.parse_expr()
        );
        assert_eq!(
            Expr {
                kind: ExprKind::Lit(Lit::Bool(false)),
                span: Span::new(29, 34),
            },
            parser.parse_expr()
        );
        assert_eq!(
            Expr {
                kind: ExprKind::Ident(identifier),
                span: Span::new(35, 45),
            },
            parser.parse_expr()
        );
    }

    #[test]
    fn parse_binary_expressions() {
        let mut parser = Parser::new("10 + -10 * 2 + 2");

        let ten = Lit::Num(Symbol::intern("10"));
        let two = Lit::Num(Symbol::intern("2"));

        let mul_two = Expr {
            kind: ExprKind::Binary(
                Box::new(Expr {
                    kind: ExprKind::Unary(
                        UnaryOp::Minus,
                        Box::new(Expr {
                            kind: ExprKind::Lit(ten.clone()),
                            span: Span::new(6, 8),
                        }),
                    ),
                    span: Span::new(5, 8),
                }),
                BinOp::Mul,
                Box::new(Expr {
                    kind: ExprKind::Lit(two.clone()),
                    span: Span::new(11, 12),
                }),
            ),
            span: Span::new(5, 12),
        };

        let add_two = Expr {
            kind: ExprKind::Binary(
                Box::new(mul_two),
                BinOp::Add,
                Box::new(Expr {
                    kind: ExprKind::Lit(two),
                    span: Span::new(15, 16),
                }),
            ),
            span: Span::new(5, 16),
        };

        assert_eq!(
            Expr {
                kind: ExprKind::Binary(
                    Box::new(Expr {
                        kind: ExprKind::Lit(ten),
                        span: Span::new(0, 2),
                    }),
                    BinOp::Add,
                    Box::new(add_two)
                ),
                span: Span::new(0, 16),
            },
            parser.parse_expr()
        );
    }
}
