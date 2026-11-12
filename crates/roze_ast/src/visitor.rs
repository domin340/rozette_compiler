use crate::{
    ExprKind, PatKind, StmtKind,
    ast_decl::{
        Block, Expr, Fn, ForLoop, Ident, If, Item, ItemKind, ItemsBlock, Label, Lit, Local,
        LocalItem, Mod, ModBlock, Param, Params, Pat, Path, PathSeg, Stmt, TyPat, TyPatKind,
        UseTree, UseTreeSuf, Vis,
    },
};

macro_rules! visit_list {
    ($name:ident.$method:ident($list:expr)) => {
        for walkable in $list {
            $name.$method(walkable);
        }
    };
}

macro_rules! visit_opt {
    ($name:ident.$method:ident($e:expr)) => {
        if let Some(e) = $e {
            $name.$method(e);
        }
    };
}

fn walk_items_blk<V: Visitor>(v: &mut V, blk: &ItemsBlock) {
    visit_list!(v.visit_item(&blk.items))
}

fn walk_mod_blk<V: Visitor>(v: &mut V, mod_blk: &ModBlock) {
    v.visit_ident(&mod_blk.ident);
    v.visit_items_blk(&mod_blk.blk)
}

fn walk_mod<V: Visitor>(v: &mut V, mod_item: &Mod) {
    match mod_item {
        Mod::Block(blk) => v.visit_mod_blk(blk),
        Mod::Include(ident) => v.visit_ident(ident),
    }
}

fn walk_blk<V: Visitor>(v: &mut V, blk: &Block) {
    visit_list!(v.visit_stmt(&blk.stmts))
}

fn walk_param<V: Visitor>(v: &mut V, param: &Param) {
    v.visit_pat(&param.pat);
    visit_opt!(v.visit_ty_pat(&param.ty))
}

fn walk_local_item<V: Visitor>(v: &mut V, local: &LocalItem) {
    v.visit_ident(&local.ident);
    v.visit_ty_pat(&local.ty);
    v.visit_expr(&local.val)
}

fn walk_params<V: Visitor>(v: &mut V, params: &Params) {
    visit_list!(v.visit_param(&params.items))
}

fn walk_fn<V: Visitor>(v: &mut V, fn_item: &Fn) {
    v.visit_ident(&fn_item.ident);
    v.visit_params(&fn_item.params);
    visit_opt!(v.visit_ty_pat(&fn_item.ret_ty));
    v.visit_blk(&fn_item.blk)
}

fn walk_use_tree_suf<V: Visitor>(v: &mut V, suf: &UseTreeSuf) {
    match suf {
        UseTreeSuf::All(_) => {}
        UseTreeSuf::As(ident, _) => v.visit_ident(ident),
        UseTreeSuf::Nested(tree, _) => visit_list!(v.visit_use_tree(tree)),
    }
}

fn walk_use_tree<V: Visitor>(v: &mut V, tree: &UseTree) {
    v.visit_path(&tree.pref);
    visit_opt!(v.visit_use_tree_suf(&tree.suf))
}

fn walk_vis<V: Visitor>(v: &mut V, vis: &Vis) {
    match vis {
        Vis::Default | Vis::Pub => {}
        Vis::Restricted { path, .. } => v.visit_path(path),
    }
}

fn walk_item<V: Visitor>(v: &mut V, item: &Item) {
    match &item.kind {
        ItemKind::Fn(fn_) => v.visit_fn(fn_),
        ItemKind::Local(local_item) => v.visit_local_item(local_item),
        ItemKind::Use(use_) => v.visit_use_tree(use_),
        ItemKind::Mod(mod_) => v.visit_mod(mod_),
    }
}

fn walk_local<V: Visitor>(v: &mut V, local: &Local) {
    v.visit_pat(&local.pat);
    visit_opt!(v.visit_ty_pat(&local.ty));
    visit_opt!(v.visit_expr(&local.val))
}

fn walk_stmt<V: Visitor>(v: &mut V, stmt: &Stmt) {
    match &stmt.kind {
        StmtKind::Expr(expr) | StmtKind::Semi(expr) => v.visit_expr(expr),
        StmtKind::Let(local) => v.visit_local(local),
        StmtKind::Item(item) => v.visit_item(item),
    }
}

fn walk_path_seg<V: Visitor>(v: &mut V, seg: &PathSeg) {
    v.visit_ident(&seg.ident)
}

fn walk_path<V: Visitor>(v: &mut V, path: &Path) {
    match path {
        Path::Simple(ident) => v.visit_ident(ident),
        Path::Segs(segs) => visit_list!(v.visit_path_seg(segs)),
    }
}

fn walk_for_loop<V: Visitor>(v: &mut V, for_loop: &ForLoop) {
    visit_opt!(v.visit_label(&for_loop.label));
    v.visit_pat(&for_loop.pat);
    v.visit_expr(&for_loop.iter);
    v.visit_blk(&for_loop.blk)
}

fn walk_if<V: Visitor>(v: &mut V, if_expr: &If) {
    v.visit_expr(&if_expr.cond);
    v.visit_blk(&if_expr.blk);
    visit_opt!(v.visit_blk(&if_expr.else_blk))
}

fn walk_ty_pat<V: Visitor>(v: &mut V, ty: &TyPat) {
    match &ty.kind {
        TyPatKind::Never | TyPatKind::Unit => {}
        TyPatKind::Path(path) => v.visit_path(path),
    }
}

fn walk_pat<V: Visitor>(v: &mut V, pat: &Pat) {
    match &pat.kind {
        PatKind::Ident(_, ident) => v.visit_ident(ident),
        PatKind::Discard => {}
    }
}

fn walk_expr<V: Visitor>(v: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Assign(lhs, rhs) => {
            v.visit_expr(lhs);
            v.visit_expr(rhs)
        }
        ExprKind::Binary(lhs, _, rhs) => {
            v.visit_expr(lhs);
            v.visit_expr(rhs)
        }
        ExprKind::Break(label, val) => {
            visit_opt!(v.visit_label(label));
            visit_opt!(v.visit_expr(val))
        }
        ExprKind::Call(lhs, more) => {
            v.visit_expr(lhs);
            visit_list!(v.visit_expr(more))
        }
        ExprKind::Cast(lhs, ty) => {
            v.visit_expr(lhs);
            v.visit_ty_pat(ty)
        }
        ExprKind::Compound(lhs, _, rhs) => {
            v.visit_expr(lhs);
            v.visit_expr(rhs)
        }
        ExprKind::Continue(label) => visit_opt!(v.visit_label(label)),
        ExprKind::For(for_loop) => v.visit_for_loop(for_loop),
        ExprKind::If(if_expr) => v.visit_if(if_expr),
        ExprKind::Lit(lit) => v.visit_lit(lit),
        ExprKind::Path(path) => v.visit_path(path),
        ExprKind::Range(lhs, rhs, _) => {
            visit_opt!(v.visit_expr(lhs));
            visit_opt!(v.visit_expr(rhs))
        }
        ExprKind::Ret(val) => visit_opt!(v.visit_expr(val)),
        ExprKind::Unary(_, rhs) => v.visit_expr(rhs),
    }
}

fn walk_label<V: Visitor>(v: &mut V, label: &Label) {
    v.visit_ident(&label.0);
}

fn walk_ident<V: Visitor>(v: &mut V, ident: &Ident) {
    _ = ident;
}

fn walk_lit<V: Visitor>(v: &mut V, lit: &Lit) {
    _ = v;
    _ = lit;
}

pub trait Visitor: Sized {
    fn visit_blk(&mut self, blk: &Block) {
        walk_blk(self, blk)
    }

    fn visit_param(&mut self, param: &Param) {
        walk_param(self, param);
    }

    fn visit_mod(&mut self, mod_item: &Mod) {
        walk_mod(self, mod_item)
    }

    fn visit_mod_blk(&mut self, mod_blk: &ModBlock) {
        walk_mod_blk(self, mod_blk)
    }

    fn visit_items_blk(&mut self, v: &ItemsBlock) {
        walk_items_blk(self, v)
    }

    fn visit_local_item(&mut self, local: &LocalItem) {
        walk_local_item(self, local)
    }

    fn visit_params(&mut self, params: &Params) {
        walk_params(self, params)
    }

    fn visit_fn(&mut self, fn_item: &Fn) {
        walk_fn(self, fn_item)
    }

    fn visit_use_tree_suf(&mut self, suf: &UseTreeSuf) {
        walk_use_tree_suf(self, suf)
    }

    fn visit_use_tree(&mut self, tree: &UseTree) {
        walk_use_tree(self, tree)
    }

    fn visit_vis(&mut self, vis: &Vis) {
        walk_vis(self, vis)
    }

    fn visit_item(&mut self, item: &Item) {
        walk_item(self, item)
    }

    fn visit_local(&mut self, local: &Local) {
        walk_local(self, local)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt)
    }

    fn visit_path_seg(&mut self, seg: &PathSeg) {
        walk_path_seg(self, seg)
    }

    fn visit_path(&mut self, path: &Path) {
        walk_path(self, path)
    }

    fn visit_for_loop(&mut self, for_loop: &ForLoop) {
        walk_for_loop(self, for_loop)
    }

    fn visit_if(&mut self, if_expr: &If) {
        walk_if(self, if_expr)
    }

    fn visit_ty_pat(&mut self, ty: &TyPat) {
        walk_ty_pat(self, ty)
    }

    fn visit_pat(&mut self, pat: &Pat) {
        walk_pat(self, pat)
    }

    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr)
    }

    fn visit_label(&mut self, label: &Label) {
        walk_label(self, label)
    }

    fn visit_ident(&mut self, ident: &Ident) {
        walk_ident(self, ident)
    }

    fn visit_lit(&mut self, lit: &Lit) {
        walk_lit(self, lit)
    }
}
