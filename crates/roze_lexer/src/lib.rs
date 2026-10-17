mod tokens;

use tokens::{CommentKind, Token, TokenKind};

#[derive(Debug)]
pub struct Lexer {}

pub fn lex_all(s: &str) -> Vec<Token> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
}
