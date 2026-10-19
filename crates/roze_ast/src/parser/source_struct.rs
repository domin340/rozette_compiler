use crate::parser::span;
use roze_lexer::Token;
use roze_span::Span;

pub struct Source<'a>(pub &'a str);

impl<'a> Source<'a> {
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.0
    }

    #[inline]
    pub const fn unwrap(self) -> &'a str {
        self.0
    }
}

impl std::ops::Index<Span> for Source<'_> {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::Index<&Token> for Source<'_> {
    type Output = str;

    fn index(&self, index: &Token) -> &Self::Output {
        &self.0[span(index)]
    }
}

impl std::ops::Index<Token> for Source<'_> {
    type Output = str;

    fn index(&self, index: Token) -> &Self::Output {
        &self.0[span(&index)]
    }
}

impl std::ops::Index<std::ops::Range<usize>> for Source<'_> {
    type Output = str;

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.0[index]
    }
}
