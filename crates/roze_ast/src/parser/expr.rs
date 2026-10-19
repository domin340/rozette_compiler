use crate::parser::Parser;
use roze_lexer::TokenKind;

use TokenKind::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    Unary,
    Product,
    Sum,
    Comp,
    Eq,
    Logical,
    Assignment,
    #[default]
    Lowest,
}

impl Prec {
    pub fn op_prec(token: TokenKind) -> Option<Self> {
        Some(match token {
            Asterisk | Slash | Percent => Self::Product,
            Lt | Gt | LtEq | GtEq => Self::Comp,
            EqEq | NotEq => Self::Eq,
            Eq | AsteriskEq | SlashEq | PercentEq | PlusEq | MinusEq => Self::Assignment,
            AmperAmper | PipePipe => Self::Logical,
            _ => return None,
        })
    }
}

impl Parser {}
