mod tokens;

use std::str::Chars;
pub use tokens::{
    CommentKind, Token, TokenKind, classify_ident, is_ident_continue, is_ident_start,
};
use unicode_ident::is_xid_continue;

pub const EOF: char = '\0';

#[derive(Debug)]
pub struct Lexer<'source> {
    s: &'source str,
    chars: Chars<'source>,
    #[cfg(debug_assertions)]
    prev: Option<char>,
}

impl<'source> Lexer<'source> {
    pub fn new(s: &'source str) -> Self {
        assert!(
            s.len() <= u32::MAX as usize,
            "source byte size exceeds expected cap: {}",
            u32::MAX
        );

        Self {
            s,
            chars: s.chars(),
            prev: None,
        }
    }

    fn pos(&self) -> usize {
        self.s.len() - self.as_str().len()
    }

    fn bump(&mut self) -> char {
        let c = self.chars.next();

        #[cfg(debug_assertions)]
        {
            self.prev = c;
        }

        c.unwrap_or(EOF)
    }

    fn skip_n(&mut self, n: usize) {
        let s = self.as_str();
        debug_assert!(n > 0 && s.len() >= n);

        #[cfg(debug_assertions)]
        {
            self.prev = s[n - 1..].chars().next();
        }

        self.chars = s[n..].chars();
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

    fn bump_index(&mut self, p: Option<usize>) {
        if let Some(n) = p {
            self.skip_n(n);
        } else {
            self.chars = "".chars();
            self.prev = Some(EOF);
        }
    }

    fn line_comment(&mut self) -> TokenKind {
        debug_assert_eq!((self.prev, self.first()), (Some('/'), '/'));
        self.bump(); // '/'

        let kind = match self.first() {
            '/' => CommentKind::Doc,
            '!' => CommentKind::File,
            _ => CommentKind::Line,
        };

        self.eat_until_newline();

        TokenKind::Comment(kind)
    }

    fn inline_comment(&mut self) -> TokenKind {
        debug_assert_eq!((self.prev, self.first()), (Some('/'), '*'));
        self.bump(); // '*'

        let bytes = self.as_str().as_bytes();
        let idx = memchr::memchr_iter(b'*', bytes)
            .find(|&index| bytes.get(index + 1).copied().unwrap_or(0) == b'/')
            .map(|found_index| found_index + 2); // consume '*/'
        self.bump_index(idx);

        let closed = idx.is_some();

        TokenKind::Comment(CommentKind::Inline { closed })
    }

    fn eat_until_newline(&mut self) {
        let bytes = self.chars.as_str().as_bytes();
        let idx = memchr::memchr2(b'\r', b'\n', bytes);
        self.bump_index(idx);
    }

    fn eat_while(&mut self, p: impl Fn(char) -> bool) {
        while p(self.first()) {
            self.bump();
        }
    }

    fn eat_ident_continue(&mut self) {
        self.eat_while(is_ident_continue);
    }

    fn eat_int(&mut self, base: u32) {
        self.eat_while(|c| c.is_digit(base) || c == '_');
    }

    fn number(&mut self, first: char) -> TokenKind {
        debug_assert!(first.is_ascii_digit());

        let radix: u32 = match (first, self.first()) {
            ('0', 'b' | 'B') => {
                self.bump(); // binary sig
                2
            }
            ('0', 'o' | 'O') => {
                self.bump(); // octal sig
                8
            }
            ('0', 'x' | 'X') => {
                self.bump(); // hex sig
                16
            }
            _ => 10,
        };

        self.eat_int(radix);

        if radix == 10 {
            // fraction
            if self.first() == '.' && self.second().is_ascii_digit() {
                self.bump(); // '.'
                self.eat_int(10);
            }

            // exponent signature
            if matches!(self.first(), 'e' | 'E') {
                self.bump();

                if matches!(self.first(), '+' | '-') {
                    self.bump();
                }
            }
        }

        self.eat_ident_continue();

        TokenKind::Num
    }

    fn string(&mut self) -> TokenKind {
        // strings can only be created with double quotes
        const QUOTE: char = '"';

        let closed = loop {
            // don't stop at new line,
            // this is still a part of double quote string.
            match self.first() {
                EOF => break false,
                '\\' => self.skip_n(2), // skip this and one after it
                c if c == QUOTE => {
                    self.bump();
                    break true;
                }
                _ => {
                    self.bump();
                }
            }
        };

        TokenKind::Str { closed }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_space();

        let start = self.pos();
        let kind = match self.bump() {
            EOF => TokenKind::Eof,
            ';' => TokenKind::Semi,
            ',' => TokenKind::Comma,
            c @ ('\r' | '\n') => {
                if c == '\r' && self.first() == '\n' {
                    self.bump(); // skip windows new line style
                }

                TokenKind::NewLine
            }
            '.' => match (self.first(), self.second()) {
                ('.', '=') => {
                    self.skip_n(2);
                    TokenKind::DotDotEq
                }
                ('.', _) => {
                    self.bump();
                    TokenKind::DotDot
                }
                _ => TokenKind::Dot,
            },
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LCurly,
            '}' => TokenKind::RCurly,
            ':' if self.first() == ':' => {
                self.bump();
                TokenKind::ColonColon
            }
            ':' => TokenKind::Colon,
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
            '<' if self.first() == '=' => {
                self.bump();
                TokenKind::LtEq
            }
            '<' => TokenKind::Lt,
            '>' if self.first() == '=' => {
                self.bump();
                TokenKind::GtEq
            }
            '>' => TokenKind::Gt,
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
                TokenKind::PlusEq
            }
            '+' => TokenKind::Plus,
            '-' => match self.first() {
                '=' => {
                    self.bump();
                    TokenKind::MinusEq
                }
                '>' => {
                    self.bump();
                    TokenKind::Arrow
                }
                _ => TokenKind::Minus,
            },
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
            digit @ ('0'..='9') => self.number(digit),
            '\'' => {
                let invalid = {
                    let c = self.first();
                    !is_ident_continue(c) || c.is_ascii_digit()
                };

                self.eat_ident_continue();
                TokenKind::Lifetime { invalid }
            }
            '"' => self.string(),
            c if is_ident_start(c) => {
                self.eat_ident_continue();
                let s = &self.s[start..self.pos()];
                classify_ident(s)
            }
            _ => TokenKind::Invalid,
        };

        Token {
            kind,
            start: start as u32,
            end: self.pos() as u32,
        }
    }
}

pub fn iter(s: &str) -> impl Iterator<Item = Token> {
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

#[cfg(test)]
mod tests {
    use crate::{
        CommentKind, Token,
        TokenKind::{self, *},
    };

    fn lex(s: &str) -> Vec<Token> {
        super::iter(s).collect()
    }

    #[inline]
    fn token(kind: TokenKind, start: u32, end: u32) -> Token {
        Token { kind, start, end }
    }

    #[test]
    fn strings() {
        assert_eq!(
            vec![
                token(Str { closed: true }, 0, 19),
                token(Str { closed: true }, 20, 34),
                token(Str { closed: false }, 35, 36),
            ],
            lex(r#""hello world \' \"" "
hello world" ""#),
        );
    }

    #[test]
    fn single_quote() {
        assert_eq!(
            vec![
                token(Lifetime { invalid: false }, 0, 6),
                token(Lifetime { invalid: true }, 7, 10),
                token(Lifetime { invalid: true }, 11, 12),
            ],
            lex("'hello '12 '")
        )
    }

    #[test]
    fn numbers() {
        assert_eq!(
            vec![
                token(Num, 0, 6),
                token(Num, 7, 11),
                token(Num, 12, 17),
                token(Num, 18, 19),
                token(Num, 20, 23),
                token(Num, 24, 31),
            ],
            lex("0x1f01 2e+2 0aaaa 0 0e2 0.2E-53"),
        );
    }

    #[test]
    fn compounds_and_symbols() {
        assert_eq!(
            vec![
                token(Plus, 0, 1),
                token(PlusEq, 2, 4),
                token(Minus, 5, 6),
                token(MinusEq, 7, 9),
                token(Asterisk, 10, 11),
                token(AsteriskEq, 12, 14),
                token(Slash, 15, 16),
                token(SlashEq, 17, 19),
                token(Percent, 20, 21),
                token(PercentEq, 22, 24),
                token(Eq, 25, 26),
                token(EqEq, 27, 29),
                token(NotEq, 30, 32),
                token(Exclamation, 33, 34),
            ],
            lex("+ += - -= * *= / /= % %= = == != !")
        )
    }

    #[test]
    fn ranges_and_dot() {
        assert_eq!(
            vec![token(DotDot, 0, 2), token(DotDotEq, 3, 6), token(Dot, 7, 8)],
            lex(".. ..= .")
        );
    }

    #[test]
    fn line_comments() {
        assert_eq!(
            vec![
                token(Comment(CommentKind::Line), 0, 14),
                token(NewLine, 14, 15),
                token(Comment(CommentKind::Doc), 15, 30),
                token(NewLine, 30, 31),
                token(Comment(CommentKind::File), 31, 47),
            ],
            lex(r#"// hello world
/// doc comment
//! file comment"#),
        );
    }

    #[test]
    fn inline_comments() {
        assert_eq!(
            vec![
                token(Comment(CommentKind::Inline { closed: true }), 0, 22),
                token(Comment(CommentKind::Inline { closed: true }), 23, 35),
                token(Comment(CommentKind::Inline { closed: false }), 36, 51),
            ],
            lex(r#"/*
block /* nothing */ /* inline */ /* unterminated"#)
        )
    }
}
