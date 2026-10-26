use crate::{Pat, PatKind, ast_decl::Mutability, parser::Parser};
use TokenKind::*;
use roze_lexer::TokenKind;

impl Parser<'_> {
    pub(crate) fn parse_pat(&mut self) -> Pat {
        let token = self.token();
        self.advance();

        let pat_kind = match token.kind {
            Mut => {
                self.expect(Ident);
                PatKind::Ident(Mutability(true), self.intern(&self.prev))
            }
            Ident => PatKind::Ident(Mutability(false), self.intern(&token)),
            Underscore => PatKind::Discard,
            _ => panic!("Unexpected token: {token} while parsing pattern"),
        };

        Pat {
            kind: pat_kind,
            span: self.record_span(token.start),
        }
    }
}
