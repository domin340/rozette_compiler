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

pub trait Visitor: Sized {
    fn visit_lit(&mut self, lit: &Lit) {
        _ = lit;
    }

    fn visit_ident(&mut self, ident: &Ident) {
        _ = ident;
    }

    fn visit_label(&mut self, label: &Label) {
        self.visit_ident(&label.0);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Assign(lhs, rhs) => {
                self.visit_expr(lhs);
                self.visit_expr(rhs)
            }
            ExprKind::Binary(lhs, _, rhs) => {
                self.visit_expr(lhs);
                self.visit_expr(rhs)
            }
            ExprKind::Break(label, val) => {
                visit_opt!(self.visit_label(label));
                visit_opt!(self.visit_expr(val))
            }
            ExprKind::Call(lhs, more) => {
                self.visit_expr(lhs);
                visit_list!(self.visit_expr(more))
            }
            ExprKind::Cast(lhs, ty) => {
                self.visit_expr(lhs);
                self.visit_ty_pat(ty)
            }
            ExprKind::Compound(lhs, _, rhs) => {
                self.visit_expr(lhs);
                self.visit_expr(rhs)
            }
            ExprKind::Continue(label) => visit_opt!(self.visit_label(label)),
            ExprKind::For(for_loop) => self.visit_for_loop(for_loop),
            ExprKind::If(if_expr) => self.visit_if(if_expr),
            ExprKind::Lit(lit) => self.visit_lit(lit),
            ExprKind::Path(path) => self.visit_path(path),
            ExprKind::Range(lhs, rhs, _) => {
                visit_opt!(self.visit_expr(lhs));
                visit_opt!(self.visit_expr(rhs))
            }
            ExprKind::Ret(val) => visit_opt!(self.visit_expr(val)),
            ExprKind::Unary(_, rhs) => self.visit_expr(rhs),
        }
    }

    fn visit_pat(&mut self, pat: &Pat) {
        match &pat.kind {
            PatKind::Ident(_, ident) => self.visit_ident(ident),
            PatKind::Discard => {}
        }
    }

    fn visit_ty_pat(&mut self, ty: &TyPat) {
        match &ty.kind {
            TyPatKind::Never | TyPatKind::Unit => {}
            TyPatKind::Path(path) => self.visit_path(path),
        }
    }

    fn visit_if(&mut self, if_expr: &If) {
        self.visit_expr(&if_expr.cond);
        self.visit_blk(&if_expr.blk);
        visit_opt!(self.visit_blk(&if_expr.else_blk))
    }

    fn visit_for_loop(&mut self, for_loop: &ForLoop) {
        visit_opt!(self.visit_label(&for_loop.label));
        self.visit_pat(&for_loop.pat);
        self.visit_expr(&for_loop.iter);
        self.visit_blk(&for_loop.blk)
    }

    fn visit_path(&mut self, path: &Path) {
        match path {
            Path::Simple(ident) => self.visit_ident(ident),
            Path::Segs(segs) => visit_list!(self.visit_path_seg(segs)),
        }
    }

    fn visit_path_seg(&mut self, seg: &PathSeg) {
        self.visit_ident(&seg.ident)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Expr(expr) | StmtKind::Semi(expr) => self.visit_expr(expr),
            StmtKind::Let(local) => self.visit_local(local),
            StmtKind::Item(item) => self.visit_item(item),
        }
    }

    fn visit_local(&mut self, local: &Local) {
        self.visit_pat(&local.pat);
        visit_opt!(self.visit_ty_pat(&local.ty));
        visit_opt!(self.visit_expr(&local.val))
    }

    fn visit_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Fn(fn_) => self.visit_fn(fn_),
            ItemKind::Local(local_item) => self.visit_local_item(local_item),
            ItemKind::Use(use_) => self.visit_use_tree(use_),
            ItemKind::Mod(mod_) => self.visit_mod(mod_),
        }
    }

    fn visit_vis(&mut self, v: &Vis) {
        match v {
            Vis::Default | Vis::Pub => {}
            Vis::Restricted { path, .. } => self.visit_path(path),
        }
    }

    fn visit_use_tree(&mut self, tree: &UseTree) {
        self.visit_path(&tree.pref);
        visit_opt!(self.visit_use_tree_suf(&tree.suf))
    }

    fn visit_use_tree_suf(&mut self, suf: &UseTreeSuf) {
        match suf {
            UseTreeSuf::All(_) => {}
            UseTreeSuf::As(ident, _) => self.visit_ident(ident),
            UseTreeSuf::Nested(tree, _) => visit_list!(self.visit_use_tree(tree)),
        }
    }

    fn visit_fn(&mut self, fn_item: &Fn) {
        self.visit_ident(&fn_item.ident);
        self.visit_params(&fn_item.params);
        visit_opt!(self.visit_ty_pat(&fn_item.ret_ty));
        self.visit_blk(&fn_item.blk)
    }

    fn visit_params(&mut self, params: &Params) {
        visit_list!(self.visit_param(&params.items))
    }

    fn visit_local_item(&mut self, local: &LocalItem) {
        self.visit_ident(&local.ident);
        self.visit_ty_pat(&local.ty);
        self.visit_expr(&local.val)
    }

    fn visit_param(&mut self, param: &Param) {
        self.visit_pat(&param.pat);
        visit_opt!(self.visit_ty_pat(&param.ty))
    }

    fn visit_blk(&mut self, blk: &Block) {
        visit_list!(self.visit_stmt(&blk.stmts))
    }

    fn visit_mod(&mut self, mod_item: &Mod) {
        match mod_item {
            Mod::Block(blk) => self.visit_mod_blk(blk),
            Mod::Include(ident) => self.visit_ident(ident),
        }
    }

    fn visit_mod_blk(&mut self, mod_blk: &ModBlock) {
        self.visit_ident(&mod_blk.ident);
        self.visit_items_blk(&mod_blk.blk)
    }

    fn visit_items_blk(&mut self, blk: &ItemsBlock) {
        visit_list!(self.visit_item(&blk.items))
    }
}
