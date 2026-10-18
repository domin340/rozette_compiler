mod symbol_interner;

pub use symbol_interner::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl From<std::ops::Range<u32>> for Span {
    fn from(value: std::ops::Range<u32>) -> Self {
        Span {
            start: value.start,
            end: value.end,
        }
    }
}

impl std::ops::Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[(index.start as usize)..(index.end as usize)]
    }
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}
