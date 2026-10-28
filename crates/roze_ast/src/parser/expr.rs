use crate::{BinOp, CompoundOp, Expr, ExprKind, Lit, UnaryOp, parser::Parser};
use TokenKind::*;
use roze_lexer::TokenKind;
use roze_span::Symbol;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    /// Call, tuple, index
    Grouping,
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
            LParen => Self::Grouping,
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
                LParen => {
                    let (args, _) = self.parse_surrounded(Comma, RParen, Self::parse_expr);
                    lhs = Expr {
                        kind: ExprKind::Call(lhs.into(), args),
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

    pub(crate) fn can_begin_expr(&self) -> bool {
        match self.token().kind {
            Ident | // identifier
            LParen | LBracket | LCurly | Lt | // group
            True | False | Str | OpenStr | Num | // literals
            Exclamation | Minus => true, // unary
            _ => false,
        }
    }

    pub(crate) fn try_parse_expr(&mut self) -> Option<Box<Expr>> {
        if self.can_begin_expr() {
            Some(self.parse_expr().into())
        } else {
            None
        }
    }

    pub(crate) fn parse_expr(&mut self) -> Expr {
        let token = self.token();

        match token.kind {
            Return => {
                self.advance();
                let kind = ExprKind::Ret(self.try_parse_expr());
                let span = self.record_span(token.start);
                Expr { kind, span }
            }
            _ => self.parse_expr_in(Prec::Lowest),
        }
    }
}

#[cfg(test)]
mod tests {
    use roze_span::{Span, Symbol};
    use thin_vec::thin_vec;

    use crate::{BinOp, Expr, ExprKind, Lit, Parser, UnaryOp, intern_all};

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
    fn parse_call_expr_2() {
        let mut parser = Parser::new("true < false()");

        // how I want to do the tests in the future.
        // assert_eq!(
        //     binary!(
        //         lit!(true),
        //         BinOp::Lt,
        //         call!(
        //             lit!(false),
        //             args![],
        //         ),
        //     ),
        //     parser.parse_expr(),
        // );

        assert_eq!(
            Expr {
                kind: ExprKind::Binary(
                    Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Bool(true)),
                        span: Span::new(0, 4),
                    }),
                    BinOp::Lt,
                    Box::new(Expr {
                        kind: ExprKind::Call(
                            Box::new(Expr {
                                kind: ExprKind::Lit(Lit::Bool(false)),
                                span: Span::new(7, 12),
                            }),
                            thin_vec![]
                        ),
                        span: Span::new(7, 14),
                    }),
                ),
                span: Span::new(0, 14),
            },
            parser.parse_expr(),
        );
    }

    #[test]
    fn parse_call_expr() {
        let (two_sym, x_sym) = intern_all!("2", "x");
        let mut parser = Parser::new("!x(2,) + 2");

        assert_eq!(
            Expr {
                kind: ExprKind::Binary(
                    Box::new(Expr {
                        kind: ExprKind::Unary(
                            UnaryOp::BitNot,
                            Box::new(Expr {
                                kind: ExprKind::Call(
                                    Box::new(Expr {
                                        kind: ExprKind::Ident(x_sym),
                                        span: Span::new(1, 2),
                                    }),
                                    thin_vec![Expr {
                                        kind: ExprKind::Lit(Lit::Num(two_sym)),
                                        span: Span::new(3, 4),
                                    }],
                                ),
                                span: Span::new(1, 6),
                            })
                        ),
                        span: Span::new(0, 6),
                    }),
                    BinOp::Add,
                    Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(two_sym)),
                        span: Span::new(9, 10),
                    }),
                ),
                span: Span::new(0, 10),
            },
            parser.parse_expr(),
        );
    }

    #[test]
    fn parse_return_expr() {
        let ten_sym = intern_all!("10");
        let mut parser = Parser::new("return 10 return");

        assert_eq!(
            Expr {
                kind: ExprKind::Ret(Some(Box::new(Expr {
                    kind: ExprKind::Lit(Lit::Num(ten_sym)),
                    span: Span::new(7, 9),
                }))),
                span: Span::new(0, 9),
            },
            parser.parse_expr(),
        );

        assert_eq!(
            Expr {
                kind: ExprKind::Ret(None),
                span: Span::new(10, 16),
            },
            parser.parse_expr(),
        )
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
