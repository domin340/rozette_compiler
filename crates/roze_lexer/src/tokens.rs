#[derive(displaydoc::Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommentKind {
    /// FileComment
    File,
    /// DocComment
    Doc,
    /// LineComment
    Line,
    /// InlineComment
    Inline,
}

#[derive(displaydoc::Display, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// `{0}`
    Comment(CommentKind),
    /// `;`
    Semi,
    /// `\n`
    NewLine,
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
        if let TokenKind::Comment(c) = self.kind {
            write!(f, "{c}")?;
        } else {
            write!(f, "{:?}", self.kind)?;
        }

        write!(f, "({},{})", self.start, self.end)
    }
}
