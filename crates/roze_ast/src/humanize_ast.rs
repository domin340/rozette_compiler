use crate::{BinOp, CompoundOp, Expr, ExprKind, Lit, UnaryOp};
use std::fmt::{self, Write};

pub trait HumanizeAst
where
    Self: Sized,
{
    type Ctx: Clone;

    fn human_fmt(&self, f: &mut fmt::Formatter<'_>, ctx: &mut Self::Ctx) -> fmt::Result;

    fn humanize(&self, options: impl Into<Self::Ctx>) -> HumanizeItem<'_, Self, Self::Ctx> {
        HumanizeItem(self, options.into())
    }

    fn to_human_string(&self, options: impl Into<Self::Ctx>) -> String {
        self.humanize(options).to_string()
    }
}

pub struct HumanizeItem<'a, T: HumanizeAst, Ctx: Clone>(&'a T, Ctx);

impl<Ctx: Clone, T: HumanizeAst<Ctx = Ctx>> fmt::Display for HumanizeItem<'_, T, Ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ctx = &mut self.1.clone();
        self.0.human_fmt(f, ctx)
    }
}

#[derive(Clone)]
pub struct HumanizeCtx {
    /// The N amount of spaces that will be written every time new scope is entered.
    pub spaces: usize,
    /// See [`HumanizeOptions::spacing`].
    /// Defines how many spaces will be added every new scope.
    pub spacing: usize,
    pub newline: &'static str,
}

impl HumanizeCtx {
    fn entab(&mut self) {
        self.spaces += self.spacing;
    }

    fn detab(&mut self) {
        self.spaces -= self.spacing;
    }

    fn new_line(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.newline)?;
        write_spaces(f, self.spaces)
    }

    fn block(
        &mut self,
        f: &mut std::fmt::Formatter,
        name: &str,
        mut block_f: impl FnMut(&mut Self, &mut fmt::Formatter) -> fmt::Result,
    ) -> fmt::Result {
        // example:
        // Unary {
        //  `-`
        //  10
        // }
        write!(f, "{name} {{")?;
        self.entab();

        self.new_line(f)?;
        block_f(self, f)?;

        self.detab();
        self.new_line(f)?;
        write!(f, "}}")
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewlineKind {
    Crlf,
    Lf,
    #[default]
    System,
}

pub struct HumanizeOptions {
    pub spacing: usize,
    pub newline: NewlineKind,
}

impl From<HumanizeOptions> for HumanizeCtx {
    fn from(value: HumanizeOptions) -> Self {
        assert!(
            value.spacing <= STATIC_SPACES.len(),
            "{} is the boundry for cheap space insertions in ctx",
            STATIC_SPACES.len(),
        );

        const UNIX: &str = "\n";
        const WINDOWS: &str = "\r\n";

        #[cfg(windows)]
        const SYSTEM: &str = WINDOWS;

        #[cfg(not(windows))]
        const SYSTEM: &str = UNIX;

        Self {
            spaces: 0,
            spacing: value.spacing,
            newline: match value.newline {
                NewlineKind::System => SYSTEM,
                NewlineKind::Lf => UNIX,
                NewlineKind::Crlf => WINDOWS,
            },
        }
    }
}

impl HumanizeAst for Expr {
    type Ctx = HumanizeCtx;

    fn human_fmt(&self, f: &mut fmt::Formatter<'_>, ctx: &mut Self::Ctx) -> fmt::Result {
        match &self.kind {
            ExprKind::Lit(lit) => match lit {
                Lit::Bool(b) => f.write_str(if *b { "true" } else { "false" }),
                Lit::Str(sym) | Lit::Num(sym) => f.write_str(sym.as_str()),
            },
            ExprKind::Ident(sym) => f.write_str(sym.as_str()),
            ExprKind::Unary(op, expr) => ctx.block(f, "Unary", |ctx, f| {
                write_unary_op(f, *op)?;
                ctx.new_line(f)?;
                expr.human_fmt(f, ctx)
            }),
            ExprKind::Binary(lhs, op, rhs) => ctx.block(f, "Binary", |ctx, f| {
                lhs.human_fmt(f, ctx)?;
                ctx.new_line(f)?;
                write_bin_op(f, *op)?;
                ctx.new_line(f)?;
                rhs.human_fmt(f, ctx)
            }),
            ExprKind::Compound(lhs, op, rhs) => ctx.block(f, "Compound", |ctx, f| {
                lhs.human_fmt(f, ctx)?;
                ctx.new_line(f)?;
                write_compound_op(f, *op)?;
                ctx.new_line(f)?;
                rhs.human_fmt(f, ctx)
            }),
            ExprKind::Assign(lhs, expr) => ctx.block(f, "Assign", |ctx, f| {
                lhs.human_fmt(f, ctx)?;
                ctx.new_line(f)?;
                expr.human_fmt(f, ctx)
            }),
        }
    }
}

fn write_new_line(f: &mut fmt::Formatter) -> fmt::Result {
    f.write_char('\n')
}

const STATIC_SPACES: &str = "                                                                ";
fn write_spaces(f: &mut fmt::Formatter, spaces: usize) -> fmt::Result {
    assert!(
        spaces <= STATIC_SPACES.len(),
        "only allowing up to {} spaces per call",
        STATIC_SPACES.len()
    );
    f.write_str(&STATIC_SPACES[0..spaces])
}

fn write_bin_op(f: &mut fmt::Formatter, op: BinOp) -> fmt::Result {
    f.write_str(match op {
        BinOp::Mul => "`*`",
        BinOp::Div => "`/`",
        BinOp::Mod => "`%`",
        BinOp::Sub => "`-`",
        BinOp::Add => "`+`",
        BinOp::Lt => "`<`",
        BinOp::LtEq => "`<=`",
        BinOp::Gt => "`>`",
        BinOp::GtEq => "`>=`",
        BinOp::EqEq => "`==`",
        BinOp::Or => "`||`",
        BinOp::And => "`&&`",
    })
}

fn write_compound_op(f: &mut fmt::Formatter, op: CompoundOp) -> fmt::Result {
    f.write_str(match op {
        CompoundOp::Mul => "`*=`",
        CompoundOp::Div => "`/=`",
        CompoundOp::Mod => "`%=`",
        CompoundOp::Add => "`+=`",
        CompoundOp::Sub => "`-=`",
    })
}

fn write_unary_op(f: &mut fmt::Formatter, op: UnaryOp) -> fmt::Result {
    f.write_str(match op {
        UnaryOp::BitNot => "`!`",
        UnaryOp::Minus => "`-`",
    })
}
