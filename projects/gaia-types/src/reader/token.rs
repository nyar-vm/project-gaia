use crate::SourcePosition;
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct Token<T> {
    pub token_type: T,
    pub position: SourcePosition,
}

impl<T> Token<T> {
    pub fn get_range(&self) -> Range<usize> {
        let start = self.position.offset;
        let end = start + self.position.length;
        start..end
    }
}
