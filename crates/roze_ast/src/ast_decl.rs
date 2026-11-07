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
pub struct Ident {
    pub name: Symbol,
    pub span: Span,
}

impl From<Ident> for Path {
    fn from(value: Ident) -> Self {
        Self::Simple(value)
    }
}

/// Stores the segment of the path
/// ```txt
/// ::hello::world
/// ^-----^^------^
///  seg1    seg2
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSeg {
    pub ident: Ident,
    /// Includes the ColonColon token.
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSegs(pub ThinVec<PathSeg>);

impl From<PathSegs> for Path {
    fn from(value: PathSegs) -> Self {
        Self::Segs(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Path {
    Simple(Ident),
    Segs(PathSegs),
}

impl Path {
    pub fn span(&self) -> Span {
        match self {
            Self::Simple(ident) => ident.span.clone(),
            Self::Segs(PathSegs(segs)) => {
                Span::new(segs[0].span.start, segs.last().unwrap().span.end)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub vis: Vis,
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeKind {
    HalfOpen,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Lit(Lit),
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
    /// (`break <lifetime>? <expr>?`)
    Break(Option<Label>, Option<Box<Expr>>),
    /// (`continue <lifetime>?`)
    Continue(Option<Label>),
    /// (`my_expr as any`)
    Cast(Box<Expr>, Box<TyPat>),
    Range(Option<Box<Expr>>, Option<Box<Expr>>, RangeKind),
    /// ```txt
    /// print("hello world!", "next")
    /// ^     ^               ^ second param in vec
    /// |     | First param stored in vec
    /// | Expr
    /// ```
    Call(Box<Expr>, ThinVec<Expr>),
    Path(Path),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mutability(pub bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(pub Ident);

impl From<Ident> for Label {
    fn from(value: Ident) -> Self {
        Self(value)
    }
}

impl Label {
    pub fn name(&self) -> &Ident {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatKind {
    Ident(Mutability, Ident),
    /// `_`
    Discard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TyPatKind {
    Path(Path),
    /// `!`
    Never,
    /// `()`
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseTreeSuf {
    /// (`as my_renamed`)
    As(Ident, Span),
    /// (`::{Hello, World}`)
    Nested(ThinVec<UseTree>, Span),
    /// (`::*`)
    All(Span),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseTree {
    pub pref: Path,
    pub suf: Option<UseTreeSuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalItemKind {
    Static(Mutability),
    Const,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalItem {
    pub kind: LocalItemKind,
    pub ident: Ident,
    pub ty: Box<TyPat>,
    pub val: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemsBlock {
    pub items: ThinVec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModBlock {
    pub ident: Ident,
    pub block: ItemsBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mod {
    /// Mod with defined body block.
    Block(Box<ModBlock>),
    /// (`mod my_script`).
    /// Used to include files just like in Rust.
    Include(Ident),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Use(Box<UseTree>),
    Fn(Box<Fn>),
    Local(Box<LocalItem>),
    Mod(Mod),
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
pub enum Vis {
    /// By default he visibility is private to file.
    Default,
    /// Just (`pub`)
    Pub,
    /// Public with optional modifier
    /// ```txt
    /// (`pub(in crate::lexer)`)
    ///   ^^^ ^-------------^
    ///   Pub Vis::Restricted
    /// ```
    Restricted {
        path: Box<Path>,
        /// Spans the entire `Restricted` visiblity
        span: Span,
        /// e.g.: (`pub(crate)`)
        shorthand: bool,
    },
}

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
    pub ident: Ident,
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
