use roze_lexer::{Lexer, Token, TokenKind};

fn lex_ignore_comments_and_nl(s: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(s);
    let mut tokens = Vec::<Token>::new();

    loop {
        let token = lexer.next_token();
        match token.kind {
            TokenKind::Comment(_) | TokenKind::NewLine => continue,
            TokenKind::Eof => {
                tokens.push(token);
                break;
            }
            _ => tokens.push(token),
        }
    }

    tokens
}

pub struct TokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl TokenStream {
    pub fn new(s: &str) -> Self {
        let tokens = lex_ignore_comments_and_nl(s);
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
    pub fn nth(&self, n: usize) -> Token {
        self.get(self.index + n)
    }
}
