#![doc = include_str!("readme.md")]

use std::collections::HashMap;

pub mod custom_sections;

/// DWARF 调试信息的主要容器
#[derive(Debug, Clone)]
pub struct DwarfInfo {
    /// 编译单元列表
    pub compilation_units: Vec<CompilationUnit>,
    /// 行号程序
    pub line_programs: Vec<LineProgram>,
    /// 字符串表
    pub string_table: StringTable,
    /// 缩写表
    pub abbreviation_table: AbbreviationTable,
    /// 地址范围表
    pub address_ranges: Vec<AddressRange>,
    /// 位置列表
    pub location_lists: Vec<LocationList>,
    /// 公共名称表
    pub public_names: Vec<PublicName>,
    /// 公共类型表
    pub public_types: Vec<PublicType>,
}

/// 编译单元
#[derive(Debug, Clone)]
pub struct CompilationUnit {
    /// 单元长度
    pub length: u32,
    /// DWARF 版本
    pub version: u16,
    /// 缩写表偏移
    pub abbrev_offset: u32,
    /// 地址大小
    pub address_size: u8,
    /// 调试信息条目
    pub debug_info_entries: Vec<DebugInfoEntry>,
}

/// 调试信息条目 (DIE - Debug Information Entry)
#[derive(Debug, Clone)]
pub struct DebugInfoEntry {
    /// 缩写代码
    pub abbrev_code: u32,
    /// 标签
    pub tag: DwarfTag,
    /// 是否有子节点
    pub has_children: bool,
    /// 属性列表
    pub attributes: Vec<Attribute>,
}

/// DWARF 标签
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum DwarfTag {
    CompileUnit,
    Subprogram,
    Variable,
    FormalParameter,
    LexicalBlock,
    BaseType,
    PointerType,
    ArrayType,
    StructureType,
    UnionType,
    EnumerationType,
    Typedef,
    Member,
    Inheritance,
    Namespace,
    ImportedDeclaration,
    Unknown(u32),
}

/// 属性
#[derive(Debug, Clone)]
pub struct Attribute {
    /// 属性名
    pub name: DwarfAttribute,
    /// 属性值
    pub value: AttributeValue,
}

/// DWARF 属性名
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum DwarfAttribute {
    Name,
    LowPc,
    HighPc,
    Language,
    Producer,
    CompDir,
    StmtList,
    Type,
    ByteSize,
    BitSize,
    BitOffset,
    DataMemberLocation,
    Location,
    External,
    Declaration,
    Encoding,
    FrameBase,
    CallFile,
    CallLine,
    CallColumn,
    Unknown(u32),
}

/// 属性值
#[derive(Debug, Clone)]
pub enum AttributeValue {
    /// 地址
    Address(u32),
    /// 常量
    Constant(u64),
    /// 字符串
    String(String),
    /// 字符串表引用
    StringRef(u32),
    /// 标志
    Flag(bool),
    /// 引用
    Reference(u32),
    /// 表达式
    Expression(Vec<u8>),
    /// 块
    Block(Vec<u8>),
}

/// 行号程序
#[derive(Debug, Clone)]
pub struct LineProgram {
    /// 程序头
    pub header: LineProgramHeader,
    /// 行号表
    pub line_table: Vec<LineTableEntry>,
}

/// 行号程序头
#[derive(Debug, Clone)]
pub struct LineProgramHeader {
    /// 单元长度
    pub unit_length: u32,
    /// DWARF 版本
    pub version: u16,
    /// 头长度
    pub header_length: u32,
    /// 最小指令长度
    pub minimum_instruction_length: u8,
    /// 默认 is_stmt 值
    pub default_is_stmt: bool,
    /// 行基数
    pub line_base: i8,
    /// 行范围
    pub line_range: u8,
    /// 操作码基数
    pub opcode_base: u8,
    /// 标准操作码长度
    pub standard_opcode_lengths: Vec<u8>,
    /// 包含目录
    pub include_directories: Vec<String>,
    /// 文件名
    pub file_names: Vec<FileEntry>,
}

/// 文件条目
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// 文件名
    pub name: String,
    /// 目录索引
    pub directory_index: u32,
    /// 修改时间
    pub modification_time: u32,
    /// 文件大小
    pub file_size: u32,
}

/// 行表条目
#[derive(Copy, Debug, Clone)]
pub struct LineTableEntry {
    /// 地址
    pub address: u32,
    /// 文件索引
    pub file: u32,
    /// 行号
    pub line: u32,
    /// 列号
    pub column: u32,
    /// 是否为语句开始
    pub is_stmt: bool,
    /// 是否为基本块开始
    pub basic_block: bool,
    /// 是否为序列结束
    pub end_sequence: bool,
}

/// 字符串表
#[derive(Debug, Clone)]
pub struct StringTable {
    /// 字符串数据
    pub strings: Vec<u8>,
    /// 字符串索引映射
    pub index_map: HashMap<u32, String>,
}

/// 缩写表
#[derive(Debug, Clone)]
pub struct AbbreviationTable {
    /// 缩写条目
    pub entries: HashMap<u32, AbbreviationEntry>,
}

/// 缩写条目
#[derive(Debug, Clone)]
pub struct AbbreviationEntry {
    /// 代码
    pub code: u32,
    /// 标签
    pub tag: DwarfTag,
    /// 是否有子节点
    pub has_children: bool,
    /// 属性规范
    pub attribute_specs: Vec<AttributeSpec>,
}

/// 属性规范
#[derive(Copy, Debug, Clone)]
pub struct AttributeSpec {
    /// 属性名
    pub name: DwarfAttribute,
    /// 表单
    pub form: DwarfForm,
}

/// DWARF 表单
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum DwarfForm {
    Addr,
    Block2,
    Block4,
    Data2,
    Data4,
    Data8,
    String,
    Block,
    Block1,
    Data1,
    Flag,
    Sdata,
    Strp,
    Udata,
    RefAddr,
    Ref1,
    Ref2,
    Ref4,
    Ref8,
    RefUdata,
    Indirect,
    SecOffset,
    Exprloc,
    FlagPresent,
    RefSig8,
    Unknown(u32),
}

/// 地址范围
#[derive(Debug, Clone)]
pub struct AddressRange {
    /// 编译单元偏移
    pub cu_offset: u32,
    /// 地址大小
    pub address_size: u8,
    /// 段大小
    pub segment_size: u8,
    /// 范围列表
    pub ranges: Vec<Range>,
}

/// 范围
#[derive(Copy, Debug, Clone)]
pub struct Range {
    /// 起始地址
    pub start: u32,
    /// 结束地址
    pub end: u32,
}

/// 位置列表
#[derive(Debug, Clone)]
pub struct LocationList {
    /// 偏移
    pub offset: u32,
    /// 位置条目
    pub entries: Vec<LocationEntry>,
}

/// 位置条目
#[derive(Debug, Clone)]
pub struct LocationEntry {
    /// 起始地址
    pub start: u32,
    /// 结束地址
    pub end: u32,
    /// 位置表达式
    pub expression: Vec<u8>,
}

/// 公共名称
#[derive(Debug, Clone)]
pub struct PublicName {
    /// 名称
    pub name: String,
    /// DIE 偏移
    pub die_offset: u32,
}

/// 公共类型
#[derive(Debug, Clone)]
pub struct PublicType {
    /// 名称
    pub name: String,
    /// DIE 偏移
    pub die_offset: u32,
}

impl DwarfInfo {
    /// 创建新的 DWARF 信息
    pub fn new() -> Self {
        Self {
            compilation_units: Vec::new(),
            line_programs: Vec::new(),
            string_table: StringTable::new(),
            abbreviation_table: AbbreviationTable::new(),
            address_ranges: Vec::new(),
            location_lists: Vec::new(),
            public_names: Vec::new(),
            public_types: Vec::new(),
        }
    }

    /// 添加编译单元
    pub fn add_compilation_unit(&mut self, unit: CompilationUnit) {
        self.compilation_units.push(unit);
    }

    /// 添加行号程序
    pub fn add_line_program(&mut self, program: LineProgram) {
        self.line_programs.push(program);
    }

    /// 根据地址查找行号信息
    pub fn find_line_info(&self, address: u32) -> Option<&LineTableEntry> {
        for program in &self.line_programs {
            for entry in &program.line_table {
                if entry.address == address {
                    return Some(entry);
                }
            }
        }
        None
    }

    /// 根据名称查找公共符号
    pub fn find_public_name(&self, name: &str) -> Option<&PublicName> {
        self.public_names.iter().find(|pn| pn.name == name)
    }
}

impl StringTable {
    /// 创建新的字符串表
    pub fn new() -> Self {
        Self { strings: Vec::new(), index_map: HashMap::new() }
    }

    /// 添加字符串
    pub fn add_string(&mut self, s: &str) -> u32 {
        let offset = self.strings.len() as u32;
        self.strings.extend_from_slice(s.as_bytes());
        self.strings.push(0); // null terminator
        self.index_map.insert(offset, s.to_string());
        offset
    }

    /// 获取字符串
    pub fn get_string(&self, offset: u32) -> Option<&String> {
        self.index_map.get(&offset)
    }
}

impl AbbreviationTable {
    /// 创建新的缩写表
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// 添加缩写条目
    pub fn add_entry(&mut self, entry: AbbreviationEntry) {
        self.entries.insert(entry.code, entry);
    }

    /// 获取缩写条目
    pub fn get_entry(&self, code: u32) -> Option<&AbbreviationEntry> {
        self.entries.get(&code)
    }
}

impl Default for DwarfInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for StringTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AbbreviationTable {
    fn default() -> Self {
        Self::new()
    }
}
