pub use roze_span::{Span, Symbol};

mod ast_decl;
pub mod parser;

pub use ast_decl::{BinOp, Expr, ExprKind, Lit, Pat, PatKind, Stmt, StmtKind, UnaryOp};

/// Defines convenient constructors or ast members.
/// `ast!(<what_ast>, <kind>, <span>)`
/// E.g.: `ast!(expr, ExprKind::Dummy, Span::new(1, 2))`
#[macro_export]
macro_rules! ast {
    (expr, $kind:expr, $span:expr) => {
        Expr {
            kind: $kind,
            span: $span,
        }
    };
    (stmt, $kind:expr, $span:expr) => {
        Stmt {
            kind: $kind,
            span: $span,
        }
    };
    (pat, $kind:expr, $span:expr) => {
        Pat {
            kind: $kind,
            span: $span,
        }
    };
}
