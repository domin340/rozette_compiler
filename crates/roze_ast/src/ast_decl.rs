use thin_vec::ThinVec;

use crate::{Span, Symbol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pat {
    pub kind: PatKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TyPat {
    pub kind: TyPatKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit {
    /// Id to the string literal underlying data.
    /// Meaning, span captures the entire thing,
    /// while SymbolId only the data between quotes.
    Str(Symbol),
    Num(Symbol),
    Bool(bool),
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
    /// (`return 10`)
    Ret(Option<Box<Expr>>),
    /// (`my_expr as any`)
    Cast(Box<Expr>, Box<TyPat>),
    /// ```txt
    /// print("hello world!", "next")
    /// ^     ^               ^ second param in vec
    /// |     | First param stored in vec
    /// | Expr
    /// ```
    Call(Box<Expr>, ThinVec<Expr>),
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
pub enum TyPatKind {
    Ident(Symbol),
    /// `!`
    Never,
    /// `()`
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
    Item(Box<Item>),
    Let(Box<Local>),
    Semi(Box<Expr>),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Local {
    pub pat: Box<Pat>,
    pub ty: Option<Box<TyPat>>,
    pub val: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Fn(Box<Fn>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
}

// /// Visibility
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum Vis {
//     File,
//     Pub,
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub stmts: ThinVec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    pub items: ThinVec<Param>,
    /// Whether the params tuple has trailing comma or not.
    /// e.g.: (`x: number, y: number,`)
    pub trailing: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fn {
    // pub vis: Vis,
    pub name: Symbol,
    pub name_span: Span,
    pub params: Box<Params>,
    pub block: Block,
    pub ret_ty: Option<Box<TyPat>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub pat: Box<Pat>,
    pub ty: Option<Box<TyPat>>,
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
