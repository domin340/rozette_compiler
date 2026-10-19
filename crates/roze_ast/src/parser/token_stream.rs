use roze_lexer::{Token, TokenKind, lex};

pub struct TokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl TokenStream {
    pub fn new(s: &str) -> Self {
        let tokens = lex(s);
        debug_assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
        Self { tokens, index: 0 }
    }

    #[inline]
    fn get(&self, index: usize) -> Token {
        self.tokens.get(index).unwrap_or(&self.eof()).clone()
    }

    /// Returns the total length of all tokens lexed including Eof
    #[inline]
    pub const fn count(&self) -> usize {
        self.tokens.len()
    }

    /// Returns the last token that is ensured to be Eof token.
    #[inline]
    pub fn eof(&self) -> Token {
        self.tokens.last().unwrap().clone()
    }

    pub fn advance(&mut self) -> Token {
        let token = &self.tokens[self.index];
        self.index += 1;
        token.clone()
    }

    /// Returns token that the lexer stream is currently pointing at
    #[inline]
    pub fn cur(&self) -> Token {
        self.tokens[self.index].clone()
    }

    #[inline]
    pub fn peek(&self) -> Token {
        self.get(self.index + 1)
    }

    #[inline]
    pub fn nth(&self, n: usize) -> Token {
        self.get(self.index + n)
    }
}
