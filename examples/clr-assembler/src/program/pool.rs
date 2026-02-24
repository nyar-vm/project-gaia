use std::collections::HashMap;

/// CLR 常量池，用于存储各种元数据
#[derive(Debug, Clone, Default)]
pub struct ClrConstantPool {
    /// 字符串池 (#Strings)
    pub strings: HashMap<String, u32>,
    /// 二进制大对象池 (#Blob)
    pub blobs: HashMap<Vec<u8>, u32>,
    /// GUID 池 (#GUID)
    pub guids: HashMap<[u8; 16], u32>,
    /// 用户字符串池 (#US)
    pub user_strings: HashMap<String, u32>,
}

impl ClrConstantPool {
    /// 创建新的常量池
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加字符串到字符串池
    pub fn add_string(&mut self, s: String) -> u32 {
        let next_index = self.strings.len() as u32;
        *self.strings.entry(s).or_insert(next_index)
    }

    /// 添加二进制数据到 Blob 池
    pub fn add_blob(&mut self, b: Vec<u8>) -> u32 {
        let next_index = self.blobs.len() as u32;
        *self.blobs.entry(b).or_insert(next_index)
    }

    /// 添加 GUID 到 GUID 池
    pub fn add_guid(&mut self, g: [u8; 16]) -> u32 {
        let next_index = self.guids.len() as u32;
        *self.guids.entry(g).or_insert(next_index)
    }

    /// 添加用户字符串到用户字符串池
    pub fn add_user_string(&mut self, s: String) -> u32 {
        let next_index = self.user_strings.len() as u32;
        *self.user_strings.entry(s).or_insert(next_index)
    }
}
