use crate::SourcePosition;
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// 表示一个带有类型和源位置的令牌。
pub struct Token<T: Copy> {
    /// 令牌的类型。
    pub token_type: T,
    /// 令牌的源位置。
    pub position: SourcePosition,
}

impl<T: Copy> Token<T> {
    /// 返回令牌的源范围。
    pub fn get_range(&self) -> Range<usize> {
        let start = self.position.offset;
        let end = start + self.position.length;
        start..end
    }
}
