use crate::{Span, Symbol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit {
    /// Id to the string literal underlying data.
    /// Meaning, span captures the entire thing,
    /// while SymbolId only the data between quotes.
    Str(Symbol),
    Num(Symbol),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinOp {
    Mul,
    Div,
    Add,
    Sub,
    Mod,
    And,
    Or,
    EqEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompoundOp {
    Mul,
    Div,
    Add,
    Sub,
    Mod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp {
    Minus,
    BitNot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Lit(Lit),
    Ident(Symbol),
    /// Binary operation (`a + 2`)
    Binary(Box<Expr>, BinOp, Box<Expr>),
    /// Unary operation (`!logical`)
    Unary(UnaryOp, Box<Expr>),
    /// Compound binary operation (`a += 2`).
    Compound(Box<Expr>, CompoundOp, Box<Expr>),
    /// Assignment (`a = 2`)
    Assign(Box<Expr>, Box<Expr>),
    // TODO: Cast(Box<Expr>, Box<TyPat>)
    //       when TyPat is implemented by phase 2
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mutability(pub bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatKind {
    Ident(Mutability, Symbol),
    /// `_`
    Discard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pat {
    pub kind: PatKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
    Let(Box<Pat>, Option<Box<Expr>>),
    Semi(Box<Expr>),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}
