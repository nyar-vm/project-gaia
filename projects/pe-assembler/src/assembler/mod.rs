pub mod x64;
pub mod x86;

use crate::{
    types::{DataDirectory, DosHeader, NtHeader, OptionalHeader, PeHeader, PeProgram, PeSection, SubsystemType},
    writer::PeAssembler,
};
use gaia_types::{helpers::Architecture, GaiaError};
use pe_coff::types::CoffHeader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

// 定义 ImportTable 类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTable {
    pub dll_name: String,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTable {
    pub name: String,
    pub functions: Vec<String>,
}

// 定义编码后的指令
#[derive(Debug, Clone)]
pub struct EncodedInstruction {
    pub bytes: Vec<u8>,
    pub length: usize,
}

/// 读取配置
#[derive(Debug, Clone)]
pub struct OutputConfig {
    arch: Architecture,
}

/// PE 汇编器构建器 - 提供人体工程学友好的API
#[derive(Debug)]
pub struct PeAssemblerBuilder {
    architecture: Architecture,
    subsystem: SubsystemType,
    entry_point: Option<u32>,
    image_base: u64,
    sections: Vec<PeSection>,
    imports: HashMap<String, Vec<String>>, // DLL名 -> 函数列表
    exports: Vec<String>,
    code_data: Vec<u8>,
    data_content: Vec<u8>,
}

impl PeAssemblerBuilder {
    /// 创建新的PE汇编器构建器
    pub fn new() -> Self {
        Self {
            architecture: Architecture::X86,
            subsystem: SubsystemType::Console,
            entry_point: None,
            image_base: 0x400000,
            sections: Vec::new(),
            imports: HashMap::new(),
            exports: Vec::new(),
            code_data: Vec::new(),
            data_content: Vec::new(),
        }
    }

    /// 设置目标架构
    pub fn architecture(mut self, arch: Architecture) -> Self {
        self.architecture = arch;
        self
    }

    /// 设置子系统类型
    pub fn subsystem(mut self, subsystem: SubsystemType) -> Self {
        self.subsystem = subsystem;
        self
    }

    /// 设置入口点地址
    pub fn entry_point(mut self, entry: u32) -> Self {
        self.entry_point = Some(entry);
        self
    }

    /// 设置镜像基址
    pub fn image_base(mut self, base: u64) -> Self {
        self.image_base = base;
        self
    }

    /// 添加导入函数
    pub fn import_function(mut self, dll: &str, function: &str) -> Self {
        self.imports.entry(dll.to_string()).or_insert_with(Vec::new).push(function.to_string());
        self
    }

    /// 批量添加导入函数
    pub fn import_functions(mut self, dll: &str, functions: &[&str]) -> Self {
        let entry = self.imports.entry(dll.to_string()).or_insert_with(Vec::new);
        for func in functions {
            entry.push(func.to_string());
        }
        self
    }

    /// 添加导出函数
    pub fn export_function(mut self, function: &str) -> Self {
        self.exports.push(function.to_string());
        self
    }

    /// 设置代码数据
    pub fn code(mut self, code: Vec<u8>) -> Self {
        self.code_data = code;
        self
    }

    /// 添加数据内容
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data_content = data;
        self
    }

    /// 添加字符串数据
    pub fn string_data(mut self, text: &str) -> Self {
        let mut data = text.as_bytes().to_vec();
        data.push(0); // null terminator
        self.data_content.extend(data);
        self
    }

    /// 构建PE程序
    pub fn build(self) -> Result<PeProgram, GaiaError> {
        // 创建DOS头
        let dos_header = DosHeader {
            e_magic: 0x5A4D, // "MZ"
            e_cblp: 0x90,
            e_cp: 0x03,
            e_crlc: 0,
            e_cparhdr: 4,
            e_minalloc: 0,
            e_maxalloc: 0xFFFF,
            e_ss: 0,
            e_sp: 0xB8,
            e_csum: 0,
            e_ip: 0,
            e_cs: 0,
            e_lfarlc: 0x40,
            e_ovno: 0,
            e_res: [0; 4],
            e_oemid: 0,
            e_oeminfo: 0,
            e_res2: [0; 10],
            e_lfanew: 0x80, // PE 头偏移
        };

        // 创建NT头
        let nt_header = NtHeader {
            signature: 0x00004550, // "PE\0\0"
        };

        // 计算节数量
        let mut section_count = 0;
        if !self.code_data.is_empty() {
            section_count += 1;
        }
        if !self.data_content.is_empty() || !self.imports.is_empty() {
            section_count += 1;
        }

        // 创建COFF头
        let coff_header = CoffHeader {
            machine: match self.architecture {
                Architecture::X86 => 0x014C,
                Architecture::X86_64 => 0x8664,
                _ => 0x014C,
            },
            number_of_sections: section_count,
            time_date_stamp: 0,
            pointer_to_symbol_table: 0,
            number_of_symbols: 0,
            size_of_optional_header: 224, // PE32 可选头大小
            characteristics: 0x0102,      // IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_32BIT_MACHINE
        };

        // 创建数据目录
        let mut data_directories = vec![
            DataDirectory { virtual_address: 0, size: 0 }, // Export Table
            DataDirectory { virtual_address: 0, size: 0 }, // Import Table (稍后设置)
            DataDirectory { virtual_address: 0, size: 0 }, // Resource Table
            DataDirectory { virtual_address: 0, size: 0 }, // Exception Table
            DataDirectory { virtual_address: 0, size: 0 }, // Certificate Table
            DataDirectory { virtual_address: 0, size: 0 }, // Base Relocation Table
            DataDirectory { virtual_address: 0, size: 0 }, // Debug
            DataDirectory { virtual_address: 0, size: 0 }, // Architecture
            DataDirectory { virtual_address: 0, size: 0 }, // Global Ptr
            DataDirectory { virtual_address: 0, size: 0 }, // TLS Table
            DataDirectory { virtual_address: 0, size: 0 }, // Load Config Table
            DataDirectory { virtual_address: 0, size: 0 }, // Bound Import
            DataDirectory { virtual_address: 0, size: 0 }, // IAT (稍后设置)
            DataDirectory { virtual_address: 0, size: 0 }, // Delay Import Descriptor
            DataDirectory { virtual_address: 0, size: 0 }, // COM+ Runtime Header
            DataDirectory { virtual_address: 0, size: 0 }, // Reserved
        ];

        // 如果有导入，设置导入表和IAT
        if !self.imports.is_empty() {
            data_directories[1] = DataDirectory { virtual_address: 0x2040, size: 40 }; // Import Table
            data_directories[12] = DataDirectory { virtual_address: 0x2010, size: 16 };
            // IAT
        }

        // 创建可选头
        let optional_header = OptionalHeader {
            magic: 0x010B, // PE32
            major_linker_version: 14,
            minor_linker_version: 0,
            size_of_code: if !self.code_data.is_empty() { 0x200 } else { 0 },
            size_of_initialized_data: if !self.data_content.is_empty() || !self.imports.is_empty() { 0x200 } else { 0 },
            size_of_uninitialized_data: 0,
            address_of_entry_point: self.entry_point.unwrap_or(0x1000),
            base_of_code: 0x1000,
            base_of_data: Some(0x2000),
            image_base: self.image_base,
            section_alignment: 0x1000,
            file_alignment: 0x200,
            major_operating_system_version: 6,
            minor_operating_system_version: 0,
            major_image_version: 0,
            minor_image_version: 0,
            major_subsystem_version: 6,
            minor_subsystem_version: 0,
            win32_version_value: 0,
            size_of_image: 0x3000,
            size_of_headers: 0x200,
            checksum: 0,
            subsystem: self.subsystem,
            dll_characteristics: 0,
            size_of_stack_reserve: 0x100000,
            size_of_stack_commit: 0x1000,
            size_of_heap_reserve: 0x100000,
            size_of_heap_commit: 0x1000,
            loader_flags: 0,
            number_of_rva_and_sizes: 16,
            data_directories,
        };

        // 创建PE头
        let pe_header = PeHeader { dos_header, nt_header, coff_header, optional_header };

        let mut sections = Vec::new();

        // 创建代码节
        if !self.code_data.is_empty() {
            let mut code = self.code_data.clone();
            // 填充到文件对齐边界
            while code.len() < 0x200 {
                code.push(0);
            }

            let text_section = PeSection {
                name: ".text".to_string(),
                virtual_size: 0x1000,
                virtual_address: 0x1000,
                size_of_raw_data: 0x200,
                pointer_to_raw_data: 0x200,
                pointer_to_relocations: 0,
                pointer_to_line_numbers: 0,
                number_of_relocations: 0,
                number_of_line_numbers: 0,
                characteristics: 0x60000020, // IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ
                data: code,
            };
            sections.push(text_section);
        }

        // 创建数据节（包含导入表和数据）
        if !self.data_content.is_empty() || !self.imports.is_empty() {
            let data_section = self.build_data_section()?;
            sections.push(data_section);
        }

        // 创建导入表和导出表
        let imports = if let Some((dll_name, functions)) = self.imports.iter().next() {
            ImportTable { dll_name: dll_name.clone(), functions: functions.clone() }
        }
        else {
            ImportTable { dll_name: String::new(), functions: Vec::new() }
        };

        let exports = ExportTable { name: String::new(), functions: self.exports };

        Ok(PeProgram { header: pe_header, sections, imports, exports })
    }

    /// 构建数据节
    fn build_data_section(&self) -> Result<PeSection, GaiaError> {
        let mut data = Vec::new();

        // 添加用户数据
        data.extend_from_slice(&self.data_content);

        // 如果有导入，构建导入表
        if !self.imports.is_empty() {
            data = self.build_import_table(data)?;
        }

        // 填充到文件对齐边界
        while data.len() < 0x200 {
            data.push(0);
        }

        Ok(PeSection {
            name: ".data".to_string(),
            virtual_size: 0x1000,
            virtual_address: 0x2000,
            size_of_raw_data: 0x200,
            pointer_to_raw_data: 0x400,
            pointer_to_relocations: 0,
            pointer_to_line_numbers: 0,
            number_of_relocations: 0,
            number_of_line_numbers: 0,
            characteristics: 0xC0000040, // IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE
            data,
        })
    }

    /// 构建导入表
    fn build_import_table(&self, mut data: Vec<u8>) -> Result<Vec<u8>, GaiaError> {
        // 这里实现导入表的构建逻辑
        // 为了简化，我们只处理第一个DLL的导入
        if let Some((dll_name, functions)) = self.imports.iter().next() {
            // 导入地址表 (IAT) - 从 0x2010 开始
            while data.len() < 0x10 {
                data.push(0);
            }

            // IAT 条目
            let mut func_addr = 0x2080u32;
            for _ in functions {
                data.extend_from_slice(&func_addr.to_le_bytes());
                func_addr += 16; // 假设每个函数名占16字节
            }
            data.extend_from_slice(&[0x00; 4]); // IAT 结束标记

            // 填充到导入表区域 (0x2040)
            while data.len() < 0x40 {
                data.push(0);
            }

            // 导入表描述符
            data.extend_from_slice(&0x2060u32.to_le_bytes()); // OriginalFirstThunk
            data.extend_from_slice(&[0x00; 8]); // TimeDateStamp + ForwarderChain
            data.extend_from_slice(&0x20B0u32.to_le_bytes()); // Name (DLL名称地址)
            data.extend_from_slice(&0x2010u32.to_le_bytes()); // FirstThunk (IAT地址)

            // 导入表结束标记
            data.extend_from_slice(&[0x00; 20]);

            // 函数名指针表 (0x2060)
            func_addr = 0x2080;
            for _ in functions {
                data.extend_from_slice(&func_addr.to_le_bytes());
                func_addr += 16;
            }
            data.extend_from_slice(&[0x00; 4]); // 结束标记

            // 填充到函数名区域 (0x2080)
            while data.len() < 0x80 {
                data.push(0);
            }

            // 函数名
            for function in functions {
                data.extend_from_slice(&[0x00, 0x00]); // hint
                data.extend_from_slice(function.as_bytes());
                data.push(0); // null terminator
                              // 对齐到16字节边界
                while data.len() % 16 != 0 {
                    data.push(0);
                }
            }

            // DLL名称
            data.extend_from_slice(dll_name.as_bytes());
            data.push(0); // null terminator
        }

        Ok(data)
    }

    /// 生成PE文件字节数组
    pub fn generate(self) -> Result<Vec<u8>, GaiaError> {
        let program = self.build()?;
        let mut buffer = Cursor::default();
        let mut assembler = PeAssembler::new(&mut buffer);
        assembler.write_program(&program)?;
        Ok(buffer.into_inner())
    }
}

impl Default for PeAssemblerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
