mod tokens;

use std::str::Chars;
pub use tokens::{
    CommentKind, Token, TokenKind, classify_ident, is_ident_continue, is_ident_start,
};

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

    #[inline(always)]
    fn set_index_nth(&mut self, p: Option<usize>) {
        self.chars = match p {
            Some(index) => self.as_str()[index..].chars(),
            None => "".chars(),
        };
    }

    fn line_comment(&mut self) -> TokenKind {
        debug_assert_eq!((self.prev, self.first()), (Some('/'), '/'));
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
        debug_assert_eq!((self.prev, self.first()), (Some('/'), '*'));
        self.bump(); // '*'

        let bytes = self.as_str().as_bytes();
        let idx = memchr::memchr_iter(b'*', bytes)
            .find(|&index| bytes.get(index + 1).copied().unwrap_or(0) == b'/')
            .map(|found_index| found_index + 2); // consume '*/'
        self.set_index_nth(idx);

        let closed = idx.is_some();
        TokenKind::Comment(if closed {
            CommentKind::Inline
        } else {
            CommentKind::OpenInline
        })
    }

    fn eat_until_newline(&mut self) {
        let bytes = self.chars.as_str().as_bytes();
        let idx = memchr::memchr2(b'\r', b'\n', bytes);
        self.set_index_nth(idx);
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

    fn string(&mut self, quote: char) -> TokenKind {
        debug_assert!(matches!(quote, '"' | '\''));

        let closed = loop {
            match self.first() {
                EOF | '\r' | '\n' => break false,
                '\\' => {
                    self.bump();
                    self.bump();
                }
                c => {
                    self.bump();

                    if c == quote {
                        break true;
                    }
                }
            }
        };

        if closed {
            TokenKind::Str
        } else {
            TokenKind::OpenStr
        }
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
            '-' if self.first() == '=' => {
                self.bump();
                TokenKind::MinusEq
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
            digit @ ('0'..='9') => self.number(digit),
            quote @ ('\'' | '"') => self.string(quote),
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
    fn lex(s: &str) -> Vec<String> {
        super::iter(s).map(|t| format!("{t}")).collect()
    }

    #[test]
    fn strings() {
        assert_eq!(
            vec![
                "Str(0,16)",
                "Str(17,40)",
                "OpenStr(41,53)",
                "NewLine(53,54)",
                "OpenStr(54,75)"
            ],
            lex(r#"'Hello World \'' "double 'single' quote" "hello world
'another unterminated"#),
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            vec![
                "Num(0,6)",
                "Num(7,11)",
                "Num(12,17)",
                "Num(18,19)",
                "Num(20,23)",
                "Num(24,31)",
            ],
            lex("0x1f01 2e+2 0aaaa 0 0e2 0.2E-53"),
        );
    }

    #[test]
    fn compounds_and_symbols() {
        assert_eq!(
            vec![
                "Plus(0,1)",
                "PlusEq(2,4)",
                "Minus(5,6)",
                "MinusEq(7,9)",
                "Asterisk(10,11)",
                "AsteriskEq(12,14)",
                "Slash(15,16)",
                "SlashEq(17,19)",
                "Percent(20,21)",
                "PercentEq(22,24)",
                "Eq(25,26)",
                "EqEq(27,29)",
                "NotEq(30,32)",
                "Exclamation(33,34)",
            ],
            lex("+ += - -= * *= / /= % %= = == != !")
        )
    }

    #[test]
    fn line_comments() {
        assert_eq!(
            vec![
                "LineComment(0,14)",
                "NewLine(14,15)",
                "DocComment(15,30)",
                "NewLine(30,31)",
                "FileComment(31,47)",
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
                "InlineComment(0,22)",
                "InlineComment(23,35)",
                "OpenInlineComment(36,51)"
            ],
            lex(r#"/*
block /* nothing */ /* inline */ /* unterminated"#)
        )
    }
}
