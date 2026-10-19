mod token_stream;

use roze_lexer::Token;
use token_stream::TokenStream;

pub struct Parser {
    stream: TokenStream,
}

impl Parser {
    pub fn new(s: &str) -> Self {
        Self {
            stream: TokenStream::new(s),
        }
    }

    #[inline(always)]
    pub(super) fn token(&self) -> Token {
        self.stream.cur()
    }

    #[inline(always)]
    pub(super) fn peek_token(&self) -> Token {
        self.stream.peek()
    }
}
