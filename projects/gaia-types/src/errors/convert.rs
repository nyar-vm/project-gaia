use super::*;

/// 将GaiaErrorKind转换为GaiaError的实现
///
/// 这个转换会自动将错误种类包装到Box中，
/// 这是GaiaError结构体所要求的，可以减少内存占用
impl From<GaiaErrorKind> for GaiaError {
    fn from(value: GaiaErrorKind) -> Self {
        Self { kind: Box::new(value) }
    }
}
