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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            TokenKind::Comment(c) => write!(f, "{c:?}Comment")?,
            kind => write!(f, "{kind:?}")?,
        }

        write!(f, "({},{})", self.start, self.end)
    }
}
