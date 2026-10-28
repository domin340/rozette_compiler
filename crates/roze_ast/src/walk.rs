use crate::{Expr, ExprKind};

macro_rules! impl_walk_for_expr {
    ($vis:vis $name:ident, $ty:ty, &$($mutability:tt)?) => {
        $vis fn $name<Ctx>(e: $ty, c: &mut Ctx, f: &mut impl FnMut($ty, &mut Ctx)) {
            match &$($mutability)? e.kind {
                ExprKind::Binary(lhs, _, rhs) => {
                    $name(lhs, c, f);
                    $name(rhs, c, f);
                }
                ExprKind::Assign(lhs, rhs) => {
                    $name(lhs, c, f);
                    $name(rhs, c, f);
                }
                ExprKind::Cast(lhs, _) => $name(lhs, c, f),
                ExprKind::Call(lhs, args) => {
                    $name(lhs, c, f);
                    args.into_iter().for_each(|e| $name(e, c, f));
                }
                ExprKind::Compound(lhs, _, rhs) => {
                    $name(lhs, c, f);
                    $name(rhs, c, f);
                }
                ExprKind::Ret(lhs) => {
                    if let Some(e) = lhs {
                        $name(e, c, f);
                    }
                }
                ExprKind::Unary(_, rhs) => $name(rhs, c, f),
                ExprKind::Ident(..) | ExprKind::Lit(..) => {}
            };

            $name(e, c, f);
        }
    };
}

impl_walk_for_expr!(pub walk, &Expr, &);
impl_walk_for_expr!(pub walk_mut, &mut Expr, &mut);
