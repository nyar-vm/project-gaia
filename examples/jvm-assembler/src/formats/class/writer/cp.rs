#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CpEntry {
    Utf8(String),
    Class(u16),
    String(u16),
    Fieldref(u16, u16),
    Methodref(u16, u16),
    InterfaceMethodref(u16, u16),
    NameAndType(u16, u16),
    Integer(i32),
    Float(u32),
    Long(i64),
    Double(u64),
}
