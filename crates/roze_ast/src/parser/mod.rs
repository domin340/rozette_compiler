mod expr;
mod lit;
mod pat;
mod source_struct;
mod stmt;
mod token_stream;

use crate::{Span, parser::source_struct::Source};
use roze_lexer::{Token, TokenKind};
use token_stream::TokenStream;

#[inline(always)]
pub(super) fn span(t: &Token) -> Span {
    Span {
        start: t.start,
        end: t.end,
    }
}

pub struct Parser<'source> {
    s: Source<'source>,
    stream: TokenStream,
    /// Token previously advanced with [`TokenStream`].
    /// Initially takes [`Token::DUMMY`] value.
    prev: Token,
}

impl<'source> Parser<'source> {
    pub fn new(s: &'source str) -> Self {
        Self {
            s: Source(s),
            stream: TokenStream::new(s),
            prev: Token::DUMMY,
        }
    }

    #[inline]
    pub(super) fn is(&mut self, expected: TokenKind) -> bool {
        self.token().kind == expected
    }

    pub(super) fn expect(&mut self, expected: TokenKind) -> bool {
        let token = self.token();
        let matches_token = token.kind == expected;

        if !matches_token {
            panic!("expected {expected}, got {token}");
        }

        matches_token
    }

    #[inline]
    pub(super) fn span(&self) -> Span {
        span(&self.token())
    }

    #[inline]
    pub(super) fn record_span(&self, start: u32) -> Span {
        Span::new(start, self.prev.end)
    }

    #[inline]
    pub(super) fn token(&self) -> Token {
        self.stream.cur()
    }

    #[inline]
    pub(super) fn advance(&mut self) {
        self.prev = self.stream.advance();
    }
}
