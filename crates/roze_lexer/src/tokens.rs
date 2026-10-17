#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommentKind {
    File,
    Doc,
    Line,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Comment(CommentKind),
    Semi,
    NewLine,
    Let,
    Mut,
    As,
    Num,
    Str,
    True,
    False,
    Eq,
    EqEq,
    NotEq,
    Asterisk,
    Slash,
    Plus,
    Minus,
    Percent,
    AsteriskEq,
    SlashEq,
    PlusEq,
    MinusEq,
    PercentEq,
    Exclamation,
    PipePipe,
    AmperAmper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug)]
pub struct Lexer {}

pub fn lex_all(s: &str) -> Vec<Token> {
    todo!()
}
