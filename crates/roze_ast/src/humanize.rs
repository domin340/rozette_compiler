use roze_span::Symbol;

use crate::{BinOp, Expr, ExprKind, Lit, ast_decl::Label};
use std::fmt::{Display, Formatter, Result as FResult};

pub struct HumanDisplay<'v, Ctx: Copy, V: Sized + Human<Ctx = Ctx>> {
    v: &'v V,
    ctx: Ctx,
}

impl<'a, Ctx: Copy, V: Human<Ctx = Ctx>> Display for HumanDisplay<'a, Ctx, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        self.v.human_fmt(f, self.ctx)
    }
}

pub trait Human: Sized {
    type Ctx: Copy + Default;

    fn human_fmt(&self, f: &mut Formatter, ctx: Self::Ctx) -> FResult;

    fn to_human(&self, ctx: Self::Ctx) -> String {
        format!("{}", self.to_fmt(ctx))
    }

    fn to_human_default(&self) -> String {
        self.to_human(Self::Ctx::default())
    }

    fn to_fmt<'a>(&'a self, ctx: Self::Ctx) -> HumanDisplay<'a, Self::Ctx, Self> {
        HumanDisplay { v: self, ctx }
    }
}

impl Human for Expr {
    type Ctx = DefaultCtx;

    fn human_fmt(&self, f: &mut Formatter, ctx: Self::Ctx) -> FResult {
        expr(f, ctx, self)
    }
}

pub trait Spacer: Copy {
    /// Increments the amount of spaces in indentation level
    /// Returns the current amount of spaces given the indentation
    fn indent(&mut self);

    /// Decrements the amount of spaces in indentation level
    fn dedent(&mut self);

    /// Returns the current amount of spaces given the indentation
    fn cur_space(&self) -> usize;

    fn space(&self, f: &mut Formatter) -> FResult {
        pub const SPACE: &str = "                                                                                              ";
        let spaces = &SPACE[0..self.cur_space()];
        f.write_str(spaces)
    }

    fn nl(&self, f: &mut Formatter) -> FResult {
        f.write_str("\n")?;
        self.space(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultCtx {
    pub per_space: u16,
    pub space: u16,
}

impl Default for DefaultCtx {
    fn default() -> Self {
        Self {
            per_space: 2,
            space: 0,
        }
    }
}

impl Spacer for DefaultCtx {
    fn cur_space(&self) -> usize {
        self.space as usize
    }

    fn indent(&mut self) {
        self.space += self.per_space;
    }

    fn dedent(&mut self) {
        self.space -= self.per_space;
    }
}

pub fn blk<S: Spacer>(
    f: &mut Formatter,
    mut c: S,
    name: &'static str,
    mut inner_f: impl FnMut(&mut Formatter, S) -> FResult,
) -> FResult {
    // start the block
    write!(f, "{name} {{")?;

    c.indent();
    c.nl(f)?;
    inner_f(f, c)?;

    c.dedent();
    c.nl(f)?;
    write!(f, "}}")
}

fn sym(f: &mut Formatter, s: &Symbol) -> FResult {
    f.write_str(Symbol::as_str(s))
}

fn label(f: &mut Formatter, label: &Label) -> FResult {
    sym(f, &label.name().name)
}

fn lit(f: &mut Formatter, lit: &Lit) -> FResult {
    match lit {
        Lit::Bool(b) => f.write_str(if *b { "true" } else { "false" }),
        Lit::Num(s) | Lit::Str(s) => sym(f, s),
    }
}

fn expr<S: Spacer>(f: &mut Formatter, c: S, e: &Expr) -> FResult {
    match &e.kind {
        ExprKind::Lit(l) => lit(f, l),
        ExprKind::Binary(lhs, op, rhs) => blk(f, c, "Binary", |f, c| {
            expr(f, c, lhs)?;
            c.nl(f)?;
            write!(f, "{op:?}")?;
            c.nl(f)?;
            expr(f, c, rhs)
        }),
        ExprKind::Unary(op, lhs) => blk(f, c, "Unary", |f, c| {
            write!(f, "{op:?}")?;
            c.nl(f)?;
            expr(f, c, lhs)
        }),
        ExprKind::Assign(lhs, rhs) => blk(f, c, "Assign", |f, c| {
            expr(f, c, lhs)?;
            c.nl(f)?;
            expr(f, c, rhs)
        }),
        ExprKind::Ret(Some(v)) => blk(f, c, "Ret", |f, c| expr(f, c, v)),
        ExprKind::Ret(None) => f.write_str("Ret"),
        ExprKind::Break(l, e) => blk(f, c, "Break", |f, c| {
            if let Some(l) = l {
                label(f, l)?;
                c.nl(f)?;
            }
            if let Some(e) = e {
                expr(f, c, e)?;
                c.nl(f)?;
            }
            Ok(())
        }),
        _ => todo!(),
    }
}
