use url::Url;

#[derive(Copy, Clone, Debug)]
pub struct BinaryReader {}

#[derive(Copy, Clone, Debug)]
pub struct SourcePosition {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
    pub length: usize,
}

#[derive(Clone, Debug)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
    pub url: Option<Url>,
}