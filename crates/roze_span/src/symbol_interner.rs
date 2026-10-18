//! Exposes methods to internalize string data using one global, thread local string interner.
//! The division of ast and span was picked up from `rustc` repository.
//! This way the interner can be used across the program.

use lasso::{Key, Rodeo};
use std::cell::RefCell;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol(u32);

unsafe impl Key for Symbol {
    fn try_from_usize(int: usize) -> Option<Self> {
        assert!(
            int <= u32::MAX as usize,
            "Interner only allows up to {} interned symbols",
            u32::MAX,
        );
        Some(Self(int as u32))
    }

    fn into_usize(self) -> usize {
        self.0 as usize
    }
}

impl Symbol {
    #[inline]
    pub fn intern(s: &str) -> Self {
        G_INTERNER.with_borrow_mut(|v| v.get_or_intern(s))
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        G_INTERNER.with_borrow(|v| unsafe { std::mem::transmute::<&str, &str>(v.resolve(self)) })
    }
}

thread_local! {
    static G_INTERNER: RefCell<Rodeo<Symbol>> = RefCell::new(Rodeo::new());
}
