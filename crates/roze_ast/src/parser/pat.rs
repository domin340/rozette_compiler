use crate::{Pat, PatKind, parser::Parser};
use TokenKind::*;
use roze_lexer::TokenKind;

impl Parser<'_> {
    pub(crate) fn parse_pat(&mut self) -> Pat {
        let token = self.token();
        self.advance();

        let pat_kind = match token.kind {
            Ident => {
                let sym = self.intern(&token);
                PatKind::Ident(sym)
            }
            Underscore => PatKind::Discard,
            _ => panic!("Unexpected token: {token} while parsing pattern"),
        };

        Pat {
            kind: pat_kind,
            span: self.record_span(token.start),
        }
    }
}
