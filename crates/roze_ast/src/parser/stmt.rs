use crate::{
    Pat, Stmt, StmtKind,
    ast_decl::{Block, Fn, Local, Param, Params, Path, PathSeg, PathSegs, UseTree, UseTreeSuf},
    parser::{Parser, span},
};
use TokenKind::*;
use roze_lexer::TokenKind;
use roze_span::Span;
use thin_vec::{ThinVec, thin_vec};

impl Parser<'_> {
    /// `(leniently_parsed_path, trailing_colon_colon_token)`
    pub(crate) fn parse_path_lenient(&mut self) -> (Path, bool) {
        let start = self.token();
        if start.kind == Ident && self.stream.nth(1).kind != ColonColon {
            self.advance();
            return (Path::Simple(self.parse_ident(&start)), false);
        }

        let mut segs = thin_vec![];

        if start.kind == Ident {
            self.advance();
            segs.push(PathSeg {
                ident: self.parse_ident(&start),
                span: span(&start),
            });
        }

        let mut trailing_colon = false;

        while let token = self.token()
            && token.kind == ColonColon
        {
            self.advance(); // ColonColon

            if !self.is(Ident) {
                trailing_colon = true;
                break;
            }

            self.advance(); // Ident
            segs.push(PathSeg {
                ident: self.parse_ident(&self.prev),
                span: Span::new(token.start, self.prev.end),
            });
        }

        (PathSegs(segs).into(), trailing_colon)
    }

    fn parse_use_tree(&mut self) -> UseTree {
        let (pref, trailing_colon) = self.parse_path_lenient();

        let suf_token = self.token();
        let suf = match suf_token.kind {
            As => {
                self.advance();
                self.expect(Ident);
                Some(UseTreeSuf::As(
                    Span::new(suf_token.start, self.prev.end),
                    self.parse_ident(&self.prev),
                ))
            }
            Asterisk if trailing_colon => {
                let start = self.prev.start; // ColonColon token
                self.advance();
                Some(UseTreeSuf::All(self.record_span(start)))
            }
            LCurly if trailing_colon => {
                let start = self.prev.start; // ColonColon token
                self.advance();
                let (items, _) = self.parse_surrounded(Comma, RCurly, Self::parse_use_tree);
                Some(UseTreeSuf::Nested(items, self.record_span(start)))
            }
            k => {
                if trailing_colon {
                    panic!("Expected <ident> while parsing use tree. Found {k}");
                }

                None
            }
        };

        self.expect(Semi);

        let span = self.record_span(pref.span().start);
        UseTree { pref, suf, span }
    }

    pub(crate) fn parse_stmt(&mut self) -> Stmt {
        let token = self.token();

        let stmt_kind = match token.kind {
            Use => {
                self.advance();

                let tree = self.parse_use_tree();
                self.expect(Semi);

                StmtKind::Use(tree.into())
            }
            kw @ (Const | Let) => {
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

                let local = Box::new(Local { pat, val, ty });
                match kw {
                    Let => StmtKind::Let(local),
                    Const => StmtKind::Const(local),
                    k => unreachable!("{k} is not guarded in stmt Let|Const."),
                }
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

                let fn_struct = Fn {
                    ident: self.parse_ident(&name_token),
                    params: params.into(),
                    ret_ty,
                    block,
                };

                StmtKind::Fn(fn_struct.into())
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
        ast_decl::{
            Block, Fn, Ident, Local, Mutability, Params, PathSeg, PathSegs, TyPat, TyPatKind,
        },
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
                        kind: PatKind::Ident(
                            Mutability(true),
                            Ident {
                                name: a_sym,
                                span: Span::new(8, 9),
                            }
                        ),
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
                            kind: PatKind::Ident(
                                Mutability(false),
                                Ident {
                                    name: a_sym,
                                    span: Span::new(22, 23),
                                }
                            ),
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
                        kind: ExprKind::Path(
                            Ident {
                                name: a_sym,
                                span: Span::new(30, 31),
                            }
                            .into()
                        ),
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
                kind: StmtKind::Fn(Box::new(Fn {
                    ident: Ident {
                        name: name_sym,
                        span: Span::new(3, 7)
                    },
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
            },
            parser.parse_stmt(),
        );
    }

    #[test]
    fn parse_typed_let_stmt() {
        let (a, number) = intern_all!("a", "number");
        let mut parser = Parser::new("let a: ::number;");

        assert_eq!(
            Stmt {
                kind: StmtKind::Let(Box::new(Local {
                    pat: Box::new(Pat {
                        kind: PatKind::Ident(
                            Mutability(false),
                            Ident {
                                name: a,
                                span: Span::new(4, 5)
                            }
                        ),
                        span: Span::new(4, 5),
                    }),
                    ty: Some(Box::new(TyPat {
                        kind: TyPatKind::Path(
                            PathSegs(thin_vec![PathSeg {
                                ident: Ident {
                                    name: number,
                                    span: Span::new(9, 15)
                                },
                                span: Span::new(7, 15),
                            }])
                            .into()
                        ),
                        span: Span::new(7, 15)
                    })),
                    val: None,
                })),
                span: Span::new(0, 16),
            },
            parser.parse_stmt(),
        )
    }
}
