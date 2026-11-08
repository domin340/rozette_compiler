use crate::{
    BinOp, CompoundOp, Expr, ExprKind, Lit, UnaryOp,
    ast_decl::{ForLoop, Ident, If, Label, RangeKind},
    parser::Parser,
};
use TokenKind::*;
use roze_lexer::TokenKind;
use roze_span::{Span, Symbol};

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

        let expr_kind = match token.kind {
            unary @ (Minus | Exclamation) => {
                self.advance();
                let op = match unary {
                    Minus => UnaryOp::Minus,
                    Exclamation => UnaryOp::BitNot,
                    _ => unreachable!("`unary` doesn't guard all cases"),
                };

                let rhs = self.parse_pratt(Prec::Unary).into();
                ExprKind::Unary(op, rhs)
            }
            Str { closed } => {
                if !closed {
                    panic!("<str> isn't closed. Terminate the string with `\"`");
                }

                ExprKind::Lit(self.parse_str_lit())
            }
            If => {
                self.advance();
                let cond = Box::new(self.parse_expr());
                let blk = self.parse_stmt_block();

                let else_blk = if self.check(Else) {
                    Some(self.parse_stmt_block())
                } else {
                    None
                };

                let if_expr = If {
                    cond,
                    blk,
                    else_blk,
                };

                ExprKind::If(if_expr.into())
            }
            Lifetime { invalid: _ } => {
                let label = self.try_parse_label();
                let loop_token = self.token();
                self.advance(); // skip the loop token (must be done)
                match loop_token.kind {
                    For => ExprKind::For(self.parse_for_loop_body(label).into()),
                    k => panic!("Expected a loop expression. Found token: {k}"),
                }
            }
            For => {
                self.advance();
                ExprKind::For(self.parse_for_loop_body(None).into())
            }
            Num => ExprKind::Lit(self.parse_num_lit()),
            val @ (True | False) => {
                self.advance();
                let bool = Lit::Bool(val == True);
                ExprKind::Lit(bool)
            }
            ColonColon | Ident => ExprKind::Path(self.parse_path()),
            k => {
                self.advance();
                panic!("unsupported token {k} while parsing lhs expression");
            }
        };

        Expr {
            kind: expr_kind,
            span: self.record_span(token.start),
        }
    }

    fn parse_for_loop_body(&mut self, label: Option<Label>) -> ForLoop {
        debug_assert_eq!(self.prev.kind, For);

        let pat = Box::new(self.parse_pat());
        self.expect(In);
        let iter = Box::new(self.parse_expr());
        let blk = self.parse_stmt_block();

        ForLoop {
            label,
            pat,
            iter,
            blk,
        }
    }

    pub(crate) fn parse_pratt(&mut self, min_prec: Prec) -> Expr {
        let start = self.token().start;
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

                    let rhs = self.parse_pratt(this_prec);
                    lhs = Expr {
                        kind: ExprKind::Binary(lhs.into(), bin_op, rhs.into()),
                        span: self.record_span(start),
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

                    let rhs = self.parse_pratt(this_prec);
                    lhs = Expr {
                        kind: ExprKind::Compound(lhs.into(), compound_op, rhs.into()),
                        span: self.record_span(start),
                    };
                }
                LParen => {
                    let (args, _) = self.parse_surrounded(Comma, RParen, Self::parse_expr);
                    lhs = Expr {
                        kind: ExprKind::Call(lhs.into(), args),
                        span: self.record_span(start),
                    };
                }
                Eq => {
                    let rhs = self.parse_pratt(this_prec);
                    lhs = Expr {
                        kind: ExprKind::Assign(lhs.into(), rhs.into()),
                        span: self.record_span(start),
                    }
                }
                k => panic!("unsupported token {k} while parsing expression"),
            }
        }

        lhs
    }

    pub(crate) fn can_begin_expr(&self) -> bool {
        // FIXME: lifetime + for
        match self.token().kind {
            Ident | ColonColon | // ident, path start
            LParen | LBracket | LCurly | Lt | // group
            True | False | Str { closed: _ } | Num | // literals
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

    pub(crate) fn try_parse_label(&mut self) -> Option<Label> {
        let token = self.token();

        if let Lifetime { invalid } = token.kind {
            self.advance();

            if invalid {
                let s = &self.s[&token];
                panic!("Invalid lifetime syntax `{s}`, try: `'<ident>`",);
            }

            // take the identifier from lifetime
            // `'hello :`
            //  ^      ^ colon after lifetime
            //  | skip this single quote
            let span = Span::new(token.start + 1, token.end);
            let name = Symbol::intern(&self.s[span.clone()]);
            self.expect(Colon);

            Some(Label(Ident { name, span }))
        } else {
            None
        }
    }

    pub(crate) fn parse_expr(&mut self) -> Expr {
        let start_token = self.token();

        let expr = match start_token.kind {
            Return => {
                self.advance();
                let kind = ExprKind::Ret(self.try_parse_expr());
                let span = self.record_span(start_token.start);
                return Expr { kind, span };
            }
            Break => {
                self.advance();
                let label = self.try_parse_label();
                let kind = ExprKind::Break(label, self.try_parse_expr());
                let span = self.record_span(start_token.start);
                return Expr { kind, span };
            }
            Continue => {
                self.advance();
                let kind = ExprKind::Continue(self.try_parse_label());
                let span = self.record_span(start_token.start);
                return Expr { kind, span };
            }
            // (RangeTo and RangeToInclusive) range expressions
            //  ^^ HalfOpen ^^ Closed
            // this block also parses RangeFull.
            k @ (DotDot | DotDotEq) => {
                self.advance();

                let range_kind = if k == DotDot {
                    RangeKind::HalfOpen
                } else {
                    RangeKind::Closed
                };

                let kind = ExprKind::Range(None, self.try_parse_expr(), range_kind);
                let span = self.record_span(start_token.start);
                return Expr { kind, span };
            }
            _ => self.parse_pratt(Prec::Lowest),
        };

        let suf_token = self.token(); // parse extra suffix
        match suf_token.kind {
            k @ (DotDot | DotDotEq) => {
                self.advance();

                let range_kind = if k == DotDot {
                    RangeKind::HalfOpen
                } else {
                    RangeKind::Closed
                };

                let rhs = self.try_parse_expr();

                Expr {
                    kind: ExprKind::Range(Some(expr.into()), rhs, range_kind),
                    span: self.record_span(start_token.start),
                }
            }
            _ => expr,
        }
    }
}

#[cfg(test)]
mod tests {
    use roze_span::{Span, Symbol};
    use thin_vec::thin_vec;

    use crate::{
        BinOp, Expr, ExprKind, Lit, Parser, UnaryOp,
        ast_decl::{Ident, Path, PathSeg, RangeKind},
        intern_all,
    };

    #[test]
    fn parse_ranges() {
        let one = intern_all!("1");
        let mut parser = Parser::new("1..1 1..=1 1..; ..1; ..=1; ..");

        assert_eq!(
            Expr {
                kind: ExprKind::Range(
                    Some(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(one)),
                        span: Span::new(0, 1),
                    })),
                    Some(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(one)),
                        span: Span::new(3, 4),
                    })),
                    RangeKind::HalfOpen
                ),
                span: Span::new(0, 4),
            },
            parser.parse_expr(),
        );

        assert_eq!(
            Expr {
                kind: ExprKind::Range(
                    Some(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(one)),
                        span: Span::new(5, 6),
                    })),
                    Some(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(one)),
                        span: Span::new(9, 10),
                    })),
                    RangeKind::Closed
                ),
                span: Span::new(5, 10),
            },
            parser.parse_expr(),
        );

        assert_eq!(
            Expr {
                kind: ExprKind::Range(
                    Some(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(one)),
                        span: Span::new(11, 12),
                    })),
                    None,
                    RangeKind::HalfOpen,
                ),
                span: Span::new(11, 14),
            },
            parser.parse_expr(),
        );
        parser.advance(); // Semi

        assert_eq!(
            Expr {
                kind: ExprKind::Range(
                    None,
                    Some(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(one)),
                        span: Span::new(18, 19),
                    })),
                    RangeKind::HalfOpen,
                ),
                span: Span::new(16, 19),
            },
            parser.parse_expr(),
        );
        parser.advance(); // Semi

        assert_eq!(
            Expr {
                kind: ExprKind::Range(
                    None,
                    Some(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(one)),
                        span: Span::new(24, 25),
                    })),
                    RangeKind::Closed,
                ),
                span: Span::new(21, 25),
            },
            parser.parse_expr(),
        );
        parser.advance();

        assert_eq!(
            Expr {
                kind: ExprKind::Range(None, None, RangeKind::HalfOpen),
                span: Span::new(27, 29),
            },
            parser.parse_expr()
        );
    }

    #[test]
    fn parse_literals_and_ident() {
        let (hello, decimal, hex, identifier) =
            intern_all!("hello", "12.3e+2", "0x3faFA", "identifier");
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
                kind: ExprKind::Path(
                    Ident {
                        name: identifier,
                        span: Span::new(35, 45)
                    }
                    .into()
                ),
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
                                        kind: ExprKind::Path(
                                            Ident {
                                                name: x_sym,
                                                span: Span::new(1, 2),
                                            }
                                            .into()
                                        ),
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
    fn parse_path_segs_glob() {
        let std = intern_all!("std");
        let mut parser = Parser::new("::std");

        assert_eq!(
            Expr {
                kind: ExprKind::Path(Path::Segs(thin_vec![PathSeg {
                    ident: Ident {
                        name: std,
                        span: Span::new(2, 5),
                    },
                    span: Span::new(0, 5)
                }])),
                span: Span::new(0, 5),
            },
            parser.parse_expr()
        );
    }

    #[test]
    fn parse_path_segs() {
        let (hello, world, b) = intern_all!("hello", "world", "b");
        let mut parser = Parser::new("hello::world::b");

        assert_eq!(
            Expr {
                kind: ExprKind::Path(Path::Segs(thin_vec![
                    PathSeg {
                        ident: Ident {
                            name: hello,
                            span: Span::new(0, 5),
                        },
                        span: Span::new(0, 5),
                    },
                    PathSeg {
                        ident: Ident {
                            name: world,
                            span: Span::new(7, 12),
                        },
                        span: Span::new(5, 12),
                    },
                    PathSeg {
                        ident: Ident {
                            name: b,
                            span: Span::new(14, 15),
                        },
                        span: Span::new(12, 15),
                    },
                ])),
                span: Span::new(0, 15),
            },
            parser.parse_expr(),
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
