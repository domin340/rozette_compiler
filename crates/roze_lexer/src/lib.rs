mod tokens;

use std::str::Chars;
use tokens::{CommentKind, Token, TokenKind};

#[derive(Debug)]
pub struct Lexer<'source> {
    s: &'source str,
    chars: Chars<'source>,
    #[cfg(debug_assertions)]
    prev: Option<char>,
}

pub const EOF: char = '\0';

impl<'source> Lexer<'source> {
    pub fn new(s: &'source str) -> Self {
        if s.len() > u32::MAX as usize {
            panic!(
                "compiler is not designed for reading files >{} bytes",
                u32::MAX
            );
        }

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
            .map(|found_index| found_index + 2); // skip '*/' characters
        self.set_index_nth(idx);

        let terminated = idx.is_some();
        TokenKind::Comment(CommentKind::Inline { terminated })
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
                EOF | '\r' | '\n' => break, // unterminated
                '\\' => {
                    self.bump();
                    self.bump();
                }
                c => {
                    self.bump();

                    if c == quote {
                        break; // terminated
                    }
                }
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
            '0'..='9' => self.number(),
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

fn is_ident_start(c: char) -> bool {
    unicode_ident::is_xid_start(c) || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    unicode_ident::is_xid_continue(c) || c == '_'
}

fn classify_ident(s: &str) -> TokenKind {
    match s.len() {
        2 => match s {
            "as" => TokenKind::As,
            _ => TokenKind::Ident,
        },
        3 => match s {
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            _ => TokenKind::Ident,
        },
        _ => TokenKind::Ident,
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

pub fn lex(s: &str) -> Vec<Token> {
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
    fn lex(s: &str) -> Vec<String> {
        super::iter(s).map(|t| format!("{t}")).collect()
    }

    #[test]
    fn strings() {
        assert_eq!(
            vec![
                "Str(0,16)",
                "Str(17,40)",
                "Str(41,53)",
                "NewLine(53,54)",
                "Str(54,75)"
            ],
            lex(r#"'Hello World \'' "double 'single' quote" "hello world
'another unterminated"#),
        );
    }

    #[test]
    fn numbers() {}

    #[test]
    fn compound_symbols() {}

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
                "InlineComment(36,51)"
            ],
            lex(r#"/*
block /* nothing */ /* inline */ /* unterminated"#)
        )
    }
}
