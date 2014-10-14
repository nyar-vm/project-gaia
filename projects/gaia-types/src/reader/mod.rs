pub struct BinaryReader {
    endian: Endian,
}

pub struct SourcePosition {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
    pub length: usize,
}

pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
    pub url: Option<Url>,
}
