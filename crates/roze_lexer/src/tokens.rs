#[derive(displaydoc::Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommentKind {
    /// <file_comment>
    File,
    /// <doc_comment>
    Doc,
    /// <line_comment>
    Line,
    /// <inline_comment>
    Inline,
    /// <open_inline_comment>
    OpenInline,
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
    /// `\n`
    NewLine,
    /// `_`
    Underscore,
    /// <ident>
    Ident,
    /// `let`
    Let,
    /// `mut`
    Mut,
    /// `as`
    As,
    /// <num>
    Num,
    /// <str>
    Str,
    /// <open_str>
    OpenStr,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            TokenKind::Comment(c) => write!(f, "{c:?}Comment")?,
            kind => write!(f, "{kind:?}")?,
        }

        write!(f, "({},{})", self.start, self.end)
    }
}

pub fn is_ident_start(c: char) -> bool {
    unicode_ident::is_xid_start(c) || c == '_'
}

pub fn is_ident_continue(c: char) -> bool {
    unicode_ident::is_xid_continue(c) || c == '_'
}

pub fn classify_ident(s: &str) -> TokenKind {
    match s.len() {
        1 if s == "_" => TokenKind::Underscore,
        2 => match s {
            "as" => TokenKind::As,
            _ => TokenKind::Ident,
        },
        3 => match s {
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            _ => TokenKind::Ident,
        },
        4 => match s {
            "true" => TokenKind::True,
            _ => TokenKind::Ident,
        },
        5 => match s {
            "false" => TokenKind::False,
            _ => TokenKind::Ident,
        },
        _ => TokenKind::Ident,
    }
}
