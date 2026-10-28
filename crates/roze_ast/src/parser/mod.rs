mod expr;
mod lit;
mod pat;
mod stmt;
mod stream_source;

use crate::{
    Span,
    parser::stream_source::{Source, TokenStream},
};
use roze_lexer::{Token, TokenKind};
use roze_span::Symbol;
use thin_vec::ThinVec;

#[inline(always)]
pub(crate) fn span(t: &Token) -> Span {
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
    pub(crate) fn is(&mut self, expected: TokenKind) -> bool {
        self.token().kind == expected
    }

    /// Returns whether the [`TokenKind`] of the current token in the [`TokenStream`] is matching with expected.
    /// If values match, the token is also advanced.
    #[inline]
    pub(crate) fn check(&mut self, expected: TokenKind) -> bool {
        let is = self.token().kind == expected;

        if is {
            self.advance();
        }

        is
    }

    /// Internalizes the data in the range of Token's span.
    /// See [`Symbol::intern`].
    #[inline]
    pub(crate) fn intern(&self, token: &Token) -> Symbol {
        let s = &self.s[token];
        Symbol::intern(s)
    }

    pub(crate) fn expect(&mut self, expected: TokenKind) -> bool {
        let token = self.token();
        let matches_token = token.kind == expected;

        if !matches_token {
            panic!("expected {expected}, got {token}");
        }

        self.advance(); // advance over the expected token
        matches_token
    }

    #[inline]
    pub(crate) fn span(&self) -> Span {
        span(&self.token())
    }

    #[inline]
    pub(crate) fn record_span(&self, start: u32) -> Span {
        Span::new(start, self.prev.end)
    }

    #[inline]
    pub(crate) fn token(&self) -> Token {
        self.stream.cur()
    }

    #[inline]
    pub(crate) fn advance(&mut self) {
        self.prev = self.stream.advance();
    }

    pub(crate) fn parse_until_terminated<T>(
        &mut self,
        sep_token: TokenKind,
        close_token: TokenKind,
        mut f: impl FnMut(&mut Self) -> T,
    ) -> (ThinVec<T>, bool) {
        let mut v = ThinVec::new();
        let mut trailing_comma = false;

        while !self.check(close_token) {
            v.push(f(self)); // parse and push the element to the thin_vec
            trailing_comma = self.check(sep_token);

            if !trailing_comma {
                self.expect(close_token);
                break;
            }
        }

        (v, trailing_comma)
    }
}
