mod tokens;

use std::str::Chars;

use tokens::{CommentKind, Token, TokenKind};

#[derive(Debug)]
pub struct Lexer<'source> {
    chars: Chars<'source>,
    len: u32,
}

pub const EOF: char = '\0';

impl<'source> Lexer<'source> {
    pub fn new(s: &'source str) -> Self {
        if s.len() > u32::MAX as usize {
            panic!("compiler is not designed for reading files >{}", u32::MAX);
        }

        Self {
            chars: s.chars(),
            len: s.len() as u32,
        }
    }

    fn pos(&self) -> u32 {
        let remaining_len = self.chars.as_str().len() as u32;
        self.len - remaining_len
    }

    fn bump(&mut self) -> char {
        self.chars.next().unwrap_or(EOF)
    }

    fn first(&mut self) -> char {
        self.chars.clone().next().unwrap_or(EOF)
    }

    fn second(&mut self) -> char {
        let mut c = self.chars.clone();
        c.next();
        c.next().unwrap_or(EOF)
    }

    fn skip_space(&mut self) {
        while matches!(self.first(), ' ' | '\t') {
            self.bump();
        }
    }

    fn as_str(&self) -> &'source str {
        self.chars.as_str()
    }

    fn line_comment(&mut self) -> TokenKind {
        debug_assert_eq!((self.first(), self.second()), ('/', '/'));
        self.bump(); // '/'
        self.bump(); // '/'

        let com_kind = match self.first() {
            '/' => CommentKind::Doc,
            '!' => CommentKind::File,
            _ => CommentKind::Line,
        };

        self.eat_until_newline();

        TokenKind::Comment(com_kind)
    }

    fn inline_comment(&mut self) -> TokenKind {
        debug_assert_eq!((self.first(), self.second()), ('/', '*'));
        self.bump();
        self.bump();

        let bytes = self.as_str().as_bytes();
        let p = memchr::memchr_iter(b'*', bytes)
            .find(|&index| bytes.get(index + 1).copied().unwrap_or(0) == b'/');

        let terminated = match p {
            Some(index) => {
                self.chars = self.as_str()[index..].chars();
                true
            }
            None => {
                self.chars = "".chars();
                false
            }
        };

        TokenKind::Comment(CommentKind::Inline { terminated })
    }

    fn eat_until_newline(&mut self) {
        let bytes = self.chars.as_str().as_bytes();
        self.chars = match memchr::memchr2(b'\r', b'\n', bytes) {
            Some(index) => self.as_str()[index..].chars(),
            _ => "".chars(),
        };
    }

    #[inline(always)]
    fn eat_while(&mut self, p: impl Fn(char) -> bool) {
        while p(self.first()) {
            self.bump();
        }
    }

    fn eat_ident_continue(&mut self) {
        self.eat_while(is_ident_continue);
    }

    fn eat_digits(&mut self, base: u32) {
        self.eat_while(|c| c.is_digit(base));
    }

    fn number(&mut self) -> TokenKind {
        // base
        self.eat_ident_continue();

        // fraction
        if self.first() == '.' && self.second().is_ascii_digit() {
            self.eat_digits(10);
        }

        // exponent signature
        if matches!(self.first(), 'e' | 'E') {
            self.bump();

            if matches!(self.first(), '+' | '-') {
                self.bump();
            }
        }

        // whatever is after that still qualifies as an identifier
        self.eat_ident_continue();

        TokenKind::Num
    }

    fn string(&mut self, quote: char) -> TokenKind {
        debug_assert!(matches!(quote, '"' | '\''));

        loop {
            match self.first() {
                EOF => break,
                '\\' => {
                    self.bump();
                    self.bump();
                }
                c if c == quote => {
                    self.bump();
                    break;
                }
                _ => _ = self.bump(),
            }
        }

        TokenKind::Str
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_space();

        let start = self.pos();
        let kind = match self.bump() {
            EOF => TokenKind::Eof,
            ';' => TokenKind::Semi,
            c @ ('\r' | '\n') => {
                if c == '\r' && self.first() == '\n' {
                    // skip windows new line style
                    self.bump();
                }

                TokenKind::NewLine
            }
            '=' if self.first() == '=' => {
                self.bump();
                TokenKind::EqEq
            }
            '=' => TokenKind::Eq,
            '!' if self.first() == '=' => {
                self.bump();
                TokenKind::NotEq
            }
            '!' => TokenKind::Exclamation,
            '*' if self.first() == '=' => {
                self.bump();
                TokenKind::AsteriskEq
            }
            '*' => TokenKind::Asterisk,
            '/' => match self.first() {
                '=' => {
                    self.bump();
                    TokenKind::SlashEq
                }
                '/' => self.line_comment(),
                '*' => self.inline_comment(),
                _ => TokenKind::Slash,
            },
            '+' if self.first() == '=' => {
                self.bump();
                TokenKind::Plus
            }
            '+' => TokenKind::Plus,
            '-' if self.first() == '-' => {
                self.bump();
                TokenKind::Minus
            }
            '-' => TokenKind::Minus,
            '%' if self.first() == '=' => {
                self.bump();
                TokenKind::PercentEq
            }
            '%' => TokenKind::Percent,
            '&' if self.first() == '&' => {
                self.bump();
                TokenKind::AmperAmper
            }
            '|' if self.first() == '|' => {
                self.bump();
                TokenKind::PipePipe
            }
            c if is_ident_start(c) => {
                self.eat_ident_continue();
                TokenKind::Ident
            }
            '0'..='9' => self.number(),
            quote @ ('\'' | '"') => self.string(quote),
            _ => TokenKind::Invalid,
        };

        let end = self.pos();
        Token { kind, start, end }
    }
}

fn is_ident_start(c: char) -> bool {
    unicode_ident::is_xid_start(c) || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    unicode_ident::is_xid_continue(c) || c == '_'
}

pub fn lex(s: &str) -> impl Iterator<Item = Token> {
    let mut lexer = Lexer::new(s);
    std::iter::from_fn(move || {
        let token = lexer.next_token();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    })
}

pub fn lex_all(s: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(s);
    let mut v = vec![];

    let mut prev_kind;

    loop {
        let token = lexer.next_token();
        prev_kind = token.kind;

        v.push(token);

        if prev_kind == TokenKind::Eof {
            break;
        }
    }

    v
}

#[cfg(test)]
mod tests {
    use super::*;
}
