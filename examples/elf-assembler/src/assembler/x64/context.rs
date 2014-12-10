//! X64 代码生成上下文管理
//!
//! 提供 X64 代码生成过程中的状态管理和上下文跟踪

use std::collections::HashMap;

/// X64 寄存器枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum X64Register {
    // 64位通用寄存器
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    RBP,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,

    // 32位寄存器
    EAX,
    EBX,
    ECX,
    EDX,
    ESI,
    EDI,
    EBP,
    ESP,
    /// R8D 32位寄存器
    R8D,
    /// R9D 32位寄存器
    R9D,
    /// R10D 32位寄存器
    R10D,
    /// R11D 32位寄存器
    R11D,
    /// R12D 32位寄存器
    R12D,
    /// R13D 32位寄存器
    R13D,
    /// R14D 32位寄存器
    R14D,
    /// R15D 32位寄存器
    R15D,
}

/// 标签信息
#[derive(Debug, Clone)]
pub struct Label {
    /// 标签名称
    pub name: String,
    /// 标签在代码中的偏移（如果已定义）
    pub offset: Option<usize>,
    /// 需要回填的位置列表
    pub fixup_locations: Vec<usize>,
}

/// 重定位信息
#[derive(Debug, Clone)]
pub struct RelocationInfo {
    /// 重定位位置
    pub offset: usize,
    /// 重定位类型
    pub reloc_type: RelocationType,
    /// 目标符号
    pub symbol: String,
}

/// 重定位类型
#[derive(Debug, Clone, Copy)]
pub enum RelocationType {
    /// 32位相对地址
    Rel32,
    /// 64位绝对地址
    Abs64,
    /// RIP相对地址
    RipRel32,
}

/// 函数调用信息
#[derive(Debug, Clone)]
pub struct FunctionCall {
    /// 函数名
    pub name: String,
    /// 调用位置
    pub call_offset: usize,
    /// 是否为导入函数
    pub is_import: bool,
}

/// X64 代码生成上下文
#[derive(Debug)]
pub struct X64Context {
    /// 生成的机器码
    pub code: Vec<u8>,

    /// 当前栈偏移（相对于 RBP）
    pub stack_offset: i32,

    /// 标签管理
    pub labels: HashMap<String, Label>,

    /// 重定位信息
    pub relocations: Vec<RelocationInfo>,

    /// 函数调用信息
    pub function_calls: Vec<FunctionCall>,

    /// 字符串常量表
    pub string_constants: HashMap<String, usize>,

    /// 寄存器使用状态
    pub register_usage: HashMap<X64Register, bool>,

    /// 当前函数的栈空间大小
    pub stack_size: u32,
}

impl X64Context {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            stack_offset: 0,
            labels: HashMap::new(),
            relocations: Vec::new(),
            function_calls: Vec::new(),
            string_constants: HashMap::new(),
            register_usage: HashMap::new(),
            stack_size: 0,
        }
    }

    /// 添加机器码字节
    pub fn emit_bytes(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    /// 获取当前代码位置
    pub fn current_position(&self) -> usize {
        self.code.len()
    }

    /// 定义标签
    pub fn define_label(&mut self, name: &str) {
        let offset = self.current_position();

        if let Some(label) = self.labels.get_mut(name) {
            // 标签已存在，设置偏移并回填
            label.offset = Some(offset);
            self.fixup_label(name, offset);
        }
        else {
            // 新标签
            self.labels
                .insert(name.to_string(), Label { name: name.to_string(), offset: Some(offset), fixup_locations: Vec::new() });
        }
    }

    /// 引用标签（用于跳转指令）
    pub fn reference_label(&mut self, name: &str) -> usize {
        let current_pos = self.current_position();

        if let Some(label) = self.labels.get_mut(name) {
            if let Some(offset) = label.offset {
                // 标签已定义，计算相对偏移
                return self.calculate_relative_offset(current_pos, offset);
            }
            else {
                // 标签未定义，记录需要回填的位置
                label.fixup_locations.push(current_pos);
            }
        }
        else {
            // 新的未定义标签
            let label = Label { name: name.to_string(), offset: None, fixup_locations: vec![current_pos] };
            self.labels.insert(name.to_string(), label);
        }

        0 // 返回占位符
    }

    /// 回填标签引用
    fn fixup_label(&mut self, name: &str, label_offset: usize) {
        if let Some(label) = self.labels.get_mut(name) {
            // 先收集需要回填的位置，避免借用冲突
            let fixup_positions: Vec<usize> = label.fixup_locations.clone();

            for fixup_pos in fixup_positions {
                let relative_offset = self.calculate_relative_offset(fixup_pos, label_offset);
                // 回填4字节相对偏移
                let bytes = (relative_offset as i32).to_le_bytes();
                for (i, &byte) in bytes.iter().enumerate() {
                    if fixup_pos + i < self.code.len() {
                        self.code[fixup_pos + i] = byte;
                    }
                }
            }

            // 清空回填位置列表
            if let Some(label) = self.labels.get_mut(name) {
                label.fixup_locations.clear();
            }
        }
    }

    /// 计算相对偏移
    fn calculate_relative_offset(&self, from: usize, to: usize) -> usize {
        if to >= from {
            to - from
        }
        else {
            // 向后跳转，需要处理负偏移
            0 // 简化处理
        }
    }

    /// 分配栈空间
    pub fn allocate_stack(&mut self, size: u32) -> i32 {
        self.stack_offset -= size as i32;
        self.stack_size = self.stack_size.max((-self.stack_offset) as u32);
        self.stack_offset
    }

    /// 添加字符串常量
    pub fn add_string_constant(&mut self, value: &str) -> usize {
        if let Some(&offset) = self.string_constants.get(value) {
            offset
        }
        else {
            let offset = self.string_constants.len() * 8; // 简化的偏移计算
            self.string_constants.insert(value.to_string(), offset);
            offset
        }
    }

    /// 添加函数调用
    pub fn add_function_call(&mut self, name: &str, is_import: bool) {
        let call_offset = self.current_position();
        self.function_calls.push(FunctionCall { name: name.to_string(), call_offset, is_import });
    }

    /// 添加重定位信息
    pub fn add_relocation(&mut self, reloc_type: RelocationType, symbol: &str) {
        let offset = self.current_position();
        self.relocations.push(RelocationInfo { offset, reloc_type, symbol: symbol.to_string() });
    }
}

impl Default for X64Context {
    fn default() -> Self {
        Self::new()
    }
}
