#[derive(displaydoc::Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommentKind {
    /// <file_comment>
    File,
    /// <doc_comment>
    Doc,
    /// <line_comment>
    Line,
    /// <inline_comment>
    Inline { closed: bool },
}

#[derive(displaydoc::Display, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// `{0}`
    Comment(CommentKind),
    /// <eof>
    Eof,
    /// <invalid>
    Invalid,
    /// `;`
    Semi,
    /// `,`
    Comma,
    /// `\n`
    NewLine,
    /// `_`
    Underscore,
    /// <ident>
    Ident,
    /// `return`
    Return,
    /// `let`
    Let,
    /// `mut`
    Mut,
    /// `use`
    Use,
    /// `const`
    Const,
    /// `break`
    Break,
    /// `static`
    Static,
    /// `continue`
    Continue,
    /// `mod`
    Mod,
    /// `as`
    As,
    /// `fn`
    Fn,
    /// `pub`
    Pub,
    /// `for`
    For,
    /// `in`
    In,
    /// `if`
    If,
    /// `else`
    Else,
    /// <num>
    Num,
    /// <str>
    Str { closed: bool },
    /// <lifetime>
    Lifetime { invalid: bool },
    /// `true`
    True,
    /// `false`
    False,
    /// `=`
    Eq,
    /// `==`
    EqEq,
    /// `!=`
    NotEq,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// `*`
    Asterisk,
    /// `/`
    Slash,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `%`
    Percent,
    /// `*=`
    AsteriskEq,
    /// `/=`
    SlashEq,
    /// `+=`
    PlusEq,
    /// `-=`
    MinusEq,
    /// `%=`
    PercentEq,
    /// `!`
    Exclamation,
    /// `||`
    PipePipe,
    /// `&&`
    AmperAmper,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{{`
    LCurly,
    /// `}}`
    RCurly,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `:`
    Colon,
    /// `->`
    Arrow,
    /// `::`
    ColonColon,
    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `..=`
    DotDotEq,
}

/// Token {{ kind: {kind}, start: {start}, end: {end} }}
#[derive(displaydoc::Display, Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: u32,
    pub end: u32,
}

impl Token {
    pub const DUMMY: Token = Token {
        kind: TokenKind::Invalid,
        start: u32::MAX,
        end: u32::MAX,
    };
}

pub fn is_ident_start(c: char) -> bool {
    unicode_ident::is_xid_start(c) || c == '_'
}

pub fn is_ident_continue(c: char) -> bool {
    unicode_ident::is_xid_continue(c) || c == '_'
}

pub fn classify_ident(s: &str) -> TokenKind {
    use TokenKind::*;

    match s.len() {
        1 if s == "_" => TokenKind::Underscore,
        2 => match s {
            "as" => As,
            "fn" => Fn,
            "in" => In,
            "if" => If,
            _ => Ident,
        },
        3 => match s {
            "let" => Let,
            "mut" => Mut,
            "use" => Use,
            "mod" => Mod,
            "pub" => Pub,
            "for" => For,
            _ => Ident,
        },
        4 => match s {
            "true" => True,
            "else" => Else,
            _ => Ident,
        },
        5 => match s {
            "false" => False,
            "const" => Const,
            "break" => Break,
            _ => Ident,
        },
        6 => match s {
            "return" => Return,
            "static" => Static,
            _ => Ident,
        },
        8 if s == "continue" => Continue,
        _ => Ident,
    }
}
