use crate::{
    Pat, PatKind,
    ast_decl::{Mutability, TyPat, TyPatKind},
    parser::Parser,
};
use TokenKind::*;
use roze_lexer::TokenKind;

impl Parser<'_> {
    pub(crate) fn parse_pat(&mut self) -> Pat {
        let token = self.token();
        self.advance();

        let pat_kind = match token.kind {
            k @ (Mut | Ident) => {
                let m = if k == Mut {
                    self.expect(Ident);
                    Mutability(true)
                } else {
                    Mutability(false)
                };

                PatKind::Ident(m, self.parse_ident(&self.prev).into())
            }
            Underscore => PatKind::Discard,
            _ => panic!("Unexpected token: {token} while parsing pattern"),
        };

        Pat {
            kind: pat_kind,
            span: self.record_span(token.start),
        }
    }

    pub(crate) fn parse_ty_pat(&mut self) -> TyPat {
        let token = self.token();
        self.advance();

        let ty_kind = match token.kind {
            ColonColon | Ident => TyPatKind::Path(self.parse_path(&token)),
            Exclamation => TyPatKind::Never,
            LParen => {
                if self.check(RParen) {
                    TyPatKind::Unit
                } else {
                    // tuple
                    todo!("type tuple hasn't been implemented")
                }
            }
            _ => panic!("Unexpected token: {token} while parsing type pattern"),
        };

        TyPat {
            kind: ty_kind,
            span: self.record_span(token.start),
        }
    }
}
