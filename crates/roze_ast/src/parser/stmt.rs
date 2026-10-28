use crate::{
    Pat, Stmt, StmtKind,
    ast_decl::{Block, Fn, Item, ItemKind, Local, Param, Params},
    parser::{Parser, span},
};
use TokenKind::*;
use roze_lexer::TokenKind;
use thin_vec::ThinVec;

impl Parser<'_> {
    pub(crate) fn parse_stmt(&mut self) -> Stmt {
        let token = self.token();

        let stmt_kind = match token.kind {
            Let => {
                self.advance();
                let pat: Box<Pat> = self.parse_pat().into();

                let ty = if self.check(Colon) {
                    Some(Box::new(self.parse_ty_pat()))
                } else {
                    None
                };

                let val = if self.check(Eq) {
                    Some(Box::new(self.parse_expr()))
                } else {
                    None
                };

                self.expect(Semi);

                StmtKind::Let(Box::new(Local { pat, val, ty }))
            }
            Fn => {
                self.advance();

                let name_token = self.token();
                self.expect(Ident);

                let params = self.parse_fn_item_params();

                // `-> number`
                let ret_ty = if self.check(Arrow) {
                    Some(Box::new(self.parse_ty_pat()))
                } else {
                    None
                };

                // { <stmt>* }
                let block = self.parse_stmt_block();

                let fn_struct = ItemKind::Fn(Box::new(Fn {
                    name: self.intern(&name_token),
                    name_span: span(&name_token),
                    params: params.into(),
                    ret_ty,
                    block,
                }));

                StmtKind::Item(Box::new(Item {
                    kind: fn_struct,
                    span: self.record_span(token.start),
                }))
            }
            _ => {
                let expr = self.parse_expr().into();
                if self.is(Semi) {
                    self.advance();
                    StmtKind::Semi(expr)
                } else {
                    StmtKind::Expr(expr)
                }
            }
        };

        Stmt {
            kind: stmt_kind,
            span: self.record_span(token.start),
        }
    }

    fn parse_fn_item_params(&mut self) -> Params {
        let token_start = self.token().start;

        self.expect(LParen);

        // `x: number, y: number)`
        //                       ^ optionally a comma can appear making it trailing
        let (items, trailing) = self.parse_surrounded(Comma, RParen, Self::parse_fn_param);

        Params {
            items,
            trailing,
            span: self.record_span(token_start),
        }
    }

    pub(crate) fn parse_fn_param(&mut self) -> Param {
        let pat = self.parse_pat().into();

        let ty = if self.check(Colon) {
            Some(Box::new(self.parse_ty_pat()))
        } else {
            None
        };

        Param { pat, ty }
    }

    pub(crate) fn parse_stmt_block(&mut self) -> Block {
        let span_start = self.token().start;
        self.expect(LCurly);

        let mut stmts = ThinVec::new();
        while !matches!(self.token().kind, Eof | RCurly) {
            stmts.push(self.parse_stmt());
        }

        // second check is done to return error when it's eof instead of right curly
        self.expect(RCurly);

        let span = self.record_span(span_start);
        Block { stmts, span }
    }
}

#[cfg(test)]
mod tests {
    use roze_span::{Span, Symbol};
    use thin_vec::thin_vec;

    use crate::{
        Expr, ExprKind, Lit, Parser, Pat, PatKind, Stmt, StmtKind,
        ast_decl::{Block, Fn, Item, ItemKind, Local, Mutability, Params, TyPat, TyPatKind},
        intern_all,
    };

    #[test]
    fn semi_and_semiless_expr() {
        let ten_sym = Symbol::intern("10");
        let mut parser = Parser::new("10; 10");

        assert_eq!(
            Stmt {
                kind: StmtKind::Semi(Box::new(Expr {
                    kind: ExprKind::Lit(Lit::Num(ten_sym)),
                    span: Span::new(0, 2),
                })),
                span: Span::new(0, 3),
            },
            parser.parse_stmt(),
        );

        assert_eq!(
            Stmt {
                kind: StmtKind::Expr(Box::new(Expr {
                    kind: ExprKind::Lit(Lit::Num(ten_sym)),
                    span: Span::new(4, 6),
                })),
                span: Span::new(4, 6),
            },
            parser.parse_stmt(),
        );
    }

    #[test]
    fn let_stmt_wildcard() {
        let ten_sym = Symbol::intern("10");
        let mut parser = Parser::new("let _ = 10;");

        assert_eq!(
            Stmt {
                kind: StmtKind::Let(Box::new(Local {
                    pat: Box::new(Pat {
                        kind: PatKind::Discard,
                        span: Span::new(4, 5),
                    }),
                    val: Some(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(ten_sym)),
                        span: Span::new(8, 10),
                    })),
                    ty: None,
                })),
                span: Span::new(0, 11)
            },
            parser.parse_stmt()
        );
    }

    #[test]
    fn let_stmt_mut_empty() {
        let a_sym = Symbol::intern("a");
        let mut parser = Parser::new("let mut a;");

        assert_eq!(
            Stmt {
                kind: StmtKind::Let(Box::new(Local {
                    pat: Box::new(Pat {
                        kind: PatKind::Ident(Mutability(true), a_sym),
                        span: Span::new(4, 9),
                    }),
                    val: None,
                    ty: None,
                })),
                span: Span::new(0, 10)
            },
            parser.parse_stmt()
        );
    }

    #[test]
    fn fn_item_arrow_return() {
        let (name_sym, a_sym, ten_sym) = intern_all!("name", "a", "10");
        let mut parser = Parser::new("fn name() -> () { let a = 10; a 10 }");

        let block = Block {
            stmts: thin_vec![
                Stmt {
                    kind: StmtKind::Let(Box::new(Local {
                        pat: Box::new(Pat {
                            kind: PatKind::Ident(Mutability(false), a_sym),
                            span: Span::new(22, 23),
                        }),
                        ty: None,
                        val: Some(Box::new(Expr {
                            kind: ExprKind::Lit(Lit::Num(ten_sym)),
                            span: Span::new(26, 28),
                        })),
                    })),
                    span: Span::new(18, 29),
                },
                Stmt {
                    kind: StmtKind::Expr(Box::new(Expr {
                        kind: ExprKind::Ident(a_sym),
                        span: Span::new(30, 31),
                    })),
                    span: Span::new(30, 31),
                },
                Stmt {
                    kind: StmtKind::Expr(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(ten_sym)),
                        span: Span::new(32, 34),
                    })),
                    span: Span::new(32, 34),
                }
            ],
            span: Span::new(16, 36),
        };

        assert_eq!(
            Stmt {
                kind: StmtKind::Item(Box::new(Item {
                    kind: ItemKind::Fn(Box::new(Fn {
                        name: name_sym,
                        name_span: Span::new(3, 7),
                        params: Box::new(Params {
                            items: thin_vec![],
                            trailing: false,
                            span: Span::new(7, 9),
                        }),
                        block,
                        ret_ty: Some(Box::new(TyPat {
                            kind: TyPatKind::Unit,
                            span: Span::new(13, 15),
                        }))
                    })),
                    span: Span::new(0, 36),
                })),
                span: Span::new(0, 36),
            },
            parser.parse_stmt(),
        );
    }
}
