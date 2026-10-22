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
