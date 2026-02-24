/// 旧版本的 CLR 头结构，用于向后兼容
#[derive(Copy, Debug, Clone)]
pub struct ClrHeader {
    /// 头的总大小（字节）
    pub cb: u32,
    /// CLR 运行时主版本号
    pub major_runtime_version: u16,
    /// CLR 运行时次版本号
    pub minor_runtime_version: u16,
    /// 元数据的相对虚拟地址
    pub metadata_rva: u32,
    /// 元数据的大小（字节）
    pub metadata_size: u32,
    /// 程序集的标志位，如是否为纯 IL 代码等
    pub flags: u32,
}

/// 旧版本的元数据头结构，用于向后兼容
#[derive(Debug, Clone)]
pub struct MetadataHeader {
    /// 魔数，通常为 0x424A5342 (BSJB)
    pub signature: u32,
    /// 元数据格式主版本
    pub major_version: u16,
    /// 元数据格式次版本
    pub minor_version: u16,
    /// 保留字段，通常为 0
    pub reserved: u32,
    /// 运行时版本字符串的长度
    pub version_length: u32,
    /// 运行时版本字符串的内容
    pub version_string: String,
    /// 元数据标志位
    pub flags: u16,
    /// 元数据流的数量
    pub streams: u16,
}

/// 旧版本的流头结构，用于向后兼容
#[derive(Debug, Clone)]
pub struct StreamHeader {
    /// 该流在元数据中的偏移量
    pub offset: u32,
    /// 流的大小（字节）
    pub size: u32,
    /// 流的名称，如 "#Strings"、"#US"、"#GUID"、"#Blob" 等
    pub name: String,
}

/// 旧版本的 .NET 程序集信息，用于向后兼容
#[derive(Debug, Clone)]
pub struct DotNetAssemblyInfo {
    /// 程序集名称
    pub name: String,
    /// 版本号，格式为 major.minor.build.revision
    pub version: String,
    /// 文化区域信息，如 "zh-CN"，null 表示中性文化
    pub culture: Option<String>,
    /// 公钥标记，用于强名称验证
    pub public_key_token: Option<String>,
    /// .NET 运行时版本，如 "v4.0.30319"
    pub runtime_version: Option<String>,
}
