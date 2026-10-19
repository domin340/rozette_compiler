use crate::{Span, Symbol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit {
    /// Id to the string literal underlying data.
    /// Meaning, span captures the entire thing,
    /// while SymbolId only the data between quotes.
    Str(Symbol),
    /// Fitting integer literals.
    /// To be integer the token may not have exponent.
    Int(i64),
    /// Possible values for 'Num':
    /// - numbers exceeding standard max bit size in luau
    /// - floats
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
    /// For simplicity the compound uses [`BinaryOp`],
    /// although the compound does not support logical operators.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatKind {
    Ident(Symbol),
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
    Empty,
    Let(Box<Pat>, Box<Expr>),
    Semi(Box<Expr>),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}
