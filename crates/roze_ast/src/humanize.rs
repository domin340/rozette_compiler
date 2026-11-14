use crate::{BinOp, Expr, ExprKind};
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
    type Ctx: Copy;

    fn human_fmt(&self, f: &mut Formatter, ctx: Self::Ctx) -> FResult;

    fn to_human(&self, ctx: Self::Ctx) -> String {
        format!("{}", self.to_fmt(ctx))
    }

    fn to_fmt<'a>(&'a self, ctx: Self::Ctx) -> HumanDisplay<'a, Self::Ctx, Self> {
        HumanDisplay { v: self, ctx }
    }
}

impl Human for Expr {
    type Ctx = DefaultCtx;

    fn human_fmt(&self, f: &mut Formatter, ctx: Self::Ctx) -> FResult {
        expr(f, self, ctx)
    }
}

pub trait Spacer: Copy {
    /// Increments the amount of spaces in indentation level
    /// Returns the current amount of spaces given the indentation
    fn indented(self) -> Self;

    /// Decrements the amount of spaces in indentation level
    fn dedented(self) -> Self;

    /// Returns the current amount of spaces given the indentation
    fn cur_space(&self) -> usize;

    fn space(&self, f: &mut Formatter) -> FResult {
        pub const SPACE: &str = "                                                                                              ";
        let spaces = &SPACE[0..self.cur_space()];
        f.write_str(spaces)
    }

    fn nl(&self, f: &mut Formatter) -> FResult {
        nl(f)?;
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

    fn indented(self) -> Self {
        Self {
            space: self.space + self.per_space,
            ..self
        }
    }

    fn dedented(self) -> Self {
        Self {
            space: self.space - self.per_space,
            ..self
        }
    }
}

pub fn nl(f: &mut Formatter) -> FResult {
    f.write_str("\n")
}

pub fn blk<S: Spacer>(
    f: &mut Formatter,
    c: S,
    name: &'static str,
    mut inner_f: impl FnMut(&mut Formatter, S) -> FResult,
) -> FResult {
    // start the block
    writeln!(f, "{name} {{")?;

    c.nl(f)?;
    inner_f(f, c.indented())?;
    c.nl(f)?;
    writeln!(f, "}}")
}

fn expr<S: Spacer>(f: &mut Formatter, e: &Expr, c: S) -> FResult {
    match &e.kind {
        ExprKind::Assign(lhs, rhs) => blk(f, c, "Assign", |f, c| {
            expr(f, lhs, c)?;
            c.nl(f)?;
            expr(f, rhs, c)
        }),
        ExprKind::Binary(lhs, op, rhs) => blk(f, c, "Binary", |f, c| {
            expr(f, lhs, c)?;
            c.nl(f)?;
            write!(f, "{op:?}")?;
            c.nl(f)?;
            expr(f, rhs, c)
        }),
        ExprKind::Ret(Some(v)) => blk(f, c, "Ret", |f, c| expr(f, v, c)),
        ExprKind::Ret(None) => f.write_str("Ret"),
        _ => todo!(),
    }
}
