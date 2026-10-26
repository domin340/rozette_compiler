use crate::{Expr, Stmt, StmtKind, parser::Parser};
use TokenKind::*;
use roze_lexer::TokenKind;

impl Parser<'_> {
    pub(crate) fn parse_stmt(&mut self) -> Stmt {
        let token = self.token();

        let stmt_kind = match token.kind {
            Let => {
                self.advance();
                let pat = self.parse_pat();

                // ty_pat here when added

                let rhs_expr: Option<Box<Expr>> = if self.is(Eq) {
                    self.advance();
                    Some(self.parse_expr().into())
                } else {
                    None
                };

                self.expect(Semi);

                StmtKind::Let(pat.into(), rhs_expr)
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
}

#[cfg(test)]
mod tests {
    use roze_span::{Span, Symbol};

    use crate::{Expr, ExprKind, Lit, Parser, Pat, PatKind, Stmt, StmtKind, ast_decl::Mutability};

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
                kind: StmtKind::Let(
                    Box::new(Pat {
                        kind: PatKind::Discard,
                        span: Span::new(4, 5),
                    }),
                    Some(Box::new(Expr {
                        kind: ExprKind::Lit(Lit::Num(ten_sym)),
                        span: Span::new(8, 10),
                    }))
                ),
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
                kind: StmtKind::Let(
                    Box::new(Pat {
                        kind: PatKind::Ident(Mutability(true), a_sym),
                        span: Span::new(4, 9),
                    }),
                    None
                ),
                span: Span::new(0, 10)
            },
            parser.parse_stmt()
        );
    }
}
