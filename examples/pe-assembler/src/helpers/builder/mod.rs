use crate::{
    formats::exe::writer::ExeWriter,
    helpers::pe_writer::PeWriter,
    types::{
        tables::{ImportEntry, ImportTable},
        CoffHeader, DataDirectory, DosHeader, NtHeader, OptionalHeader, PeHeader, PeProgram, PeSection, SubsystemType,
    },
};
use gaia_types::{helpers::Architecture, GaiaError};
use std::io::Cursor;
/// PE 汇编器构建器
#[derive(Debug)]
pub struct PeBuilder {
    architecture: Option<Architecture>,
    subsystem: Option<SubsystemType>,
    entry_point: Option<u32>,
    image_base: Option<u64>,
    imports: Vec<(String, Vec<String>)>, // (dll_name, functions)
    code: Option<Vec<u8>>,
    data: Option<Vec<u8>>,
    sections: Vec<PeSection>, // Add this field
}

impl PeBuilder {
    /// 创建新的 PE 汇编器构建器
    pub fn new() -> Self {
        Self {
            architecture: None,
            subsystem: None,
            entry_point: None,
            image_base: None,
            imports: Vec::new(),
            code: None,
            data: None,
            sections: Vec::new(), // Initialize the new field
        }
    }

    /// 设置目标架构
    pub fn architecture(mut self, arch: Architecture) -> Self {
        self.architecture = Some(arch);
        self
    }

    /// 设置子系统类型
    pub fn subsystem(mut self, subsystem: SubsystemType) -> Self {
        self.subsystem = Some(subsystem);
        self
    }

    /// 设置入口点地址
    pub fn entry_point(mut self, entry_point: u32) -> Self {
        self.entry_point = Some(entry_point);
        self
    }

    /// 设置映像基地址
    pub fn image_base(mut self, image_base: u64) -> Self {
        self.image_base = Some(image_base);
        self
    }

    /// 导入单个函数
    pub fn import_function(mut self, dll_name: &str, function_name: &str) -> Self {
        // 查找是否已存在该 DLL
        if let Some(entry) = self.imports.iter_mut().find(|(name, _)| name == dll_name) {
            entry.1.push(function_name.to_string());
        }
        else {
            self.imports.push((dll_name.to_string(), vec![function_name.to_string()]));
        }
        self
    }

    /// 导入多个函数
    pub fn import_functions(mut self, dll_name: &str, function_names: &[&str]) -> Self {
        let functions: Vec<String> = function_names.iter().map(|&s| s.to_string()).collect();

        // 查找是否已存在该 DLL
        if let Some(entry) = self.imports.iter_mut().find(|(name, _)| name == dll_name) {
            entry.1.extend(functions);
        }
        else {
            self.imports.push((dll_name.to_string(), functions));
        }
        self
    }

    /// 设置代码数据
    pub fn code(mut self, code: Vec<u8>) -> Self {
        self.code = Some(code);
        self
    }

    /// 设置数据
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// 获取导入信息（用于创建导入表）
    pub fn get_imports(&self) -> &Vec<(String, Vec<String>)> {
        &self.imports
    }

    /// 生成 PE 头部信息
    pub fn build_header(&self) -> Result<PeHeader, GaiaError> {
        let architecture = self
            .architecture
            .as_ref()
            .ok_or_else(|| GaiaError::syntax_error("Architecture is required", gaia_types::SourceLocation::default()))?;
        let pointer_size: u32 = if *architecture == Architecture::X86_64 { 8 } else { 4 };
        let subsystem = self
            .subsystem
            .ok_or_else(|| GaiaError::syntax_error("Subsystem is required", gaia_types::SourceLocation::default()))?;
        let entry_point = self.entry_point.unwrap_or(0x1000);
        let image_base = self.image_base.unwrap_or(match architecture {
            Architecture::X86 => 0x400000,
            Architecture::X86_64 => 0x140000000,
            _ => 0x400000,
        });

        // 创建 DOS 头
        let dos_header = DosHeader::new(0x80);

        // 创建 NT 头
        let nt_header = NtHeader {
            signature: 0x00004550, // "PE\0\0"
        };

        // 创建 COFF 头
        let machine = match architecture {
            Architecture::X86 => 0x014C,
            Architecture::X86_64 => 0x8664,
            _ => 0x014C,
        };

        let mut section_count = 0;
        if self.code.is_some() {
            section_count += 1;
        }
        if self.data.is_some() {
            section_count += 1;
        }
        if !self.imports.is_empty() {
            section_count += 1;
        }

        let optional_header_size = match architecture {
            Architecture::X86_64 => 240,
            _ => 224,
        };

        // 根据架构设置 COFF 特征位：
        // - x86: 可执行映像 | 32 位机器
        // - x64: 可执行映像 | 大地址感知（不设置 32 位机器位）
        let characteristics = match architecture {
            Architecture::X86 => 0x0102,    // IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_32BIT_MACHINE
            Architecture::X86_64 => 0x0022, // IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_LARGE_ADDRESS_AWARE
            _ => 0x0102,
        };

        let coff_header = CoffHeader::new(machine, section_count)
            .with_timestamp(0)
            .with_symbol_table(0, 0)
            .with_optional_header_size(optional_header_size)
            .with_characteristics(characteristics);

        // 规范化 size_of_code 和 size_of_initialized_data，使用节的对齐后大小
        let size_of_code = if let Some(code) = &self.code { ((code.len() + 0x1FF) / 0x200 * 0x200) as u32 } else { 0 };
        let mut size_of_initialized_data = 0;
        if let Some(data) = &self.data {
            size_of_initialized_data += ((data.len() + 0x1FF) / 0x200 * 0x200) as u32;
            // .data
        }
        if !self.imports.is_empty() {
            // 这里我们无法准确预测 .idata 大小，但 0x200 应该足够大多数情况
            // 或者我们可以先构建节来获取准确大小
            size_of_initialized_data += 0x200; // .idata
        }

        // 计算 size_of_image：从 0x1000 开始，每个实际存在的节增加 0x1000
        let mut size_of_image = 0x1000; // DOS/Headers 占一个对齐页
        if self.code.is_some() {
            size_of_image += 0x1000;
        }
        if self.data.is_some() {
            size_of_image += 0x1000;
        }
        if !self.imports.is_empty() {
            size_of_image += 0x1000;
        }

        let mut optional_header = OptionalHeader::new_for_architecture(
            architecture,
            entry_point,
            image_base,
            size_of_code,
            0x200,         // size_of_headers
            size_of_image, // 动态计算的镜像大小
            subsystem,
        );
        optional_header.size_of_initialized_data = size_of_initialized_data;

        // 关闭 ASLR（DYNAMIC_BASE），否则我们修补的绝对地址会因随机基址而失效
        // DYNAMIC_BASE 位值为 0x0040
        optional_header.dll_characteristics &= !0x0040;

        // 设置导入表数据目录（动态计算 RVA）
        // 兼容模式（x64）：IAT 与 INT 初始都指向 IMAGE_IMPORT_BY_NAME（Hint+Name）的 RVA。
        // - x64：INT=名称指针数组，IAT=名称RVA（加载器解析后覆盖为真实地址）
        // - x86：OFT/INT=0，IAT=名称RVA（常见兼容布局）
        if !self.imports.is_empty() {
            // 查找 .idata 节的实际 RVA
            let idata_section = self
                .sections
                .iter()
                .find(|s| s.name == ".idata")
                .ok_or_else(|| GaiaError::syntax_error("Missing .idata section", gaia_types::SourceLocation::default()))?;
            let import_rva_base = idata_section.virtual_address;

            // 后续计算保持不变，但使用 import_rva_base 代替硬编码值
            let mut current_rva = import_rva_base + ((self.imports.len() + 1) as u32) * 20;
            for (dll_name, _) in &self.imports {
                current_rva += (dll_name.len() as u32) + 1;
            }
            if current_rva % 2 != 0 {
                current_rva += 1;
            }
            // 函数 Hint/Name
            for (_, functions) in &self.imports {
                for func in functions {
                    // 名称按 2 字节对齐
                    if current_rva % 2 != 0 {
                        current_rva += 1;
                    }
                    current_rva += 2 + (func.len() as u32) + 1;
                }
            }
            // 移除这里多余的对齐，改为与 write_import_table 保持一致
            // INT
            if current_rva % pointer_size != 0 {
                current_rva = (current_rva + pointer_size - 1) & !(pointer_size - 1);
            }
            for (_, functions) in &self.imports {
                current_rva += ((functions.len() as u32) + 1) * pointer_size;
            }
            // IAT（到此处即为 IAT 起点）
            if current_rva % pointer_size != 0 {
                current_rva = (current_rva + pointer_size - 1) & !(pointer_size - 1);
            }
            let iat_rva_start = current_rva; // 记录 IAT 起点
                                             // IAT 长度
            let mut end_rva = current_rva;
            for (_, functions) in &self.imports {
                end_rva += ((functions.len() as u32) + 1) * pointer_size;
            }
            optional_header.data_directories[1] =
                DataDirectory { virtual_address: import_rva_base, size: end_rva - import_rva_base };
            // 同时填写 IAT Directory（索引 12），让加载器知道 IAT 范围
            let mut iat_rva_end = iat_rva_start;
            for (_, functions) in &self.imports {
                iat_rva_end += ((functions.len() as u32) + 1) * pointer_size;
            }
            optional_header.data_directories[12] =
                DataDirectory { virtual_address: iat_rva_start, size: iat_rva_end - iat_rva_start };
        }

        Ok(PeHeader { dos_header, nt_header, coff_header, optional_header })
    }

    /// 生成节列表
    pub fn build_sections(&mut self) -> Vec<PeSection> {
        let mut sections = Vec::new();
        let mut next_virtual_address = 0x1000;
        let mut next_raw_data_offset = 0x200;

        // 预先计算节的 RVA，以便 fix_code_relocations 使用
        // 注意：这里的逻辑必须与下面的节添加顺序完全一致
        let mut code_rva = None;
        let mut data_rva = None;

        if self.code.is_some() {
            code_rva = Some(next_virtual_address);
            next_virtual_address += 0x1000;
        }
        if self.data.is_some() {
            data_rva = Some(next_virtual_address);
            next_virtual_address += 0x1000;
        }

        // 重置以进行实际构建
        next_virtual_address = 0x1000;

        // 添加代码节
        if let Some(code) = &self.code {
            let mut code_data = code.clone();

            // 修复重定位
            self.fix_code_relocations_with_rvas(&mut code_data, code_rva.unwrap_or(0x1000), data_rva);

            // 对齐到 512 字节
            let raw_size = ((code_data.len() + 0x1FF) / 0x200 * 0x200) as u32;
            while code_data.len() < raw_size as usize {
                code_data.push(0);
            }

            let text_section = PeSection {
                name: ".text".to_string(),
                virtual_size: 0x1000,
                virtual_address: next_virtual_address,
                size_of_raw_data: raw_size,
                pointer_to_raw_data: next_raw_data_offset,
                pointer_to_relocations: 0,
                pointer_to_line_numbers: 0,
                number_of_relocations: 0,
                number_of_line_numbers: 0,
                characteristics: 0x60000020,
                data: code_data,
            };
            sections.push(text_section);
            next_virtual_address += 0x1000;
            next_raw_data_offset += raw_size;
        }

        // 添加数据节
        if let Some(data) = &self.data {
            let mut data_bytes = data.clone();
            // 对齐到 512 字节
            let raw_size = ((data_bytes.len() + 0x1FF) / 0x200 * 0x200) as u32;
            while data_bytes.len() < raw_size as usize {
                data_bytes.push(0);
            }

            let data_section = PeSection {
                name: ".data".to_string(),
                virtual_size: 0x1000,
                virtual_address: next_virtual_address,
                size_of_raw_data: raw_size,
                pointer_to_raw_data: next_raw_data_offset,
                pointer_to_relocations: 0,
                pointer_to_line_numbers: 0,
                number_of_relocations: 0,
                number_of_line_numbers: 0,
                characteristics: 0xC0000040,
                data: data_bytes,
            };
            sections.push(data_section);
            next_virtual_address += 0x1000;
            next_raw_data_offset += raw_size;
        }

        // 添加导入表节（如果有导入）
        if !self.imports.is_empty() {
            let mut idata_section = self.build_import_section();
            idata_section.virtual_address = next_virtual_address;
            idata_section.pointer_to_raw_data = next_raw_data_offset;
            sections.push(idata_section);
        }

        sections
    }

    /// 构建导入表节
    fn build_import_section(&self) -> PeSection {
        // 不在这里填充数据，让 write_import_table 方法来处理。
        // 注意：write_import_table 使用“兼容模式”（见上文），在 x64 下 IAT 初始填入 Hint/Name 的 RVA。
        PeSection {
            name: ".idata".to_string(),
            virtual_size: 0x1000,
            virtual_address: 0x3000, // 这个值会在 build_sections 中被覆盖
            size_of_raw_data: 0x200,
            pointer_to_raw_data: 0x600, // 这个值会在 build_sections 中被覆盖
            pointer_to_relocations: 0,
            pointer_to_line_numbers: 0,
            number_of_relocations: 0,
            number_of_line_numbers: 0,
            characteristics: 0xC0000040, // IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE
            data: Vec::new(),            // 空数据，由 write_import_table 方法填充
        }
    }

    /// 修复代码中的重定位
    fn fix_code_relocations_with_rvas(&self, code: &mut Vec<u8>, code_section_rva: u32, data_section_rva: Option<u32>) {
        // 查找 CALL 指令的占位符并替换为正确的地址
        let mut i = 0;

        let arch = self.architecture.as_ref().unwrap_or(&Architecture::X86).clone();
        let pointer_size: usize = if arch == Architecture::X86_64 { 8 } else { 4 };
        let image_base: u64 = self.image_base.as_ref().copied().unwrap_or(match arch {
            Architecture::X86 => 0x400000,
            Architecture::X86_64 => 0x140000000,
            _ => 0x400000,
        });

        // 计算 .idata 节的起始 RVA（与 build_sections 一致）
        let mut import_rva_base: u64 = 0x1000; // 起始 RVA
        if self.code.is_some() {
            import_rva_base += 0x1000;
        }
        if self.data.is_some() {
            import_rva_base += 0x1000;
        }

        // 计算 IAT 的实际位置（与 write_import_table 中的计算完全一致）
        let mut current_rva: u64 = import_rva_base;

        // 计算 IAT 的偏移量（跟 write_import_table 中的计算保持一致）
        if !self.imports.is_empty() {
            // 导入描述符表大小：(导入DLL数量 + 1) * 20字节
            current_rva += ((self.imports.len() + 1) * 20) as u64;

            // DLL 名称大小
            for (dll_name, _) in &self.imports {
                current_rva += (dll_name.len() + 1) as u64; // 包括空终止符
            }

            // 对齐到 2 字节，与 write_import_table 保持一致
            if current_rva % 2 != 0 {
                current_rva += 1;
            }

            // 函数名称（Hint(2) + Name + '\0'），逐个累加
            for (_, functions) in &self.imports {
                for function in functions {
                    // 名称按 2 字节对齐
                    if current_rva % 2 != 0 {
                        current_rva += 1;
                    }
                    current_rva += 2 + (function.len() + 1) as u64;
                }
            }

            // INT（OriginalFirstThunk）按 pointer_size 字节对齐并分配
            if current_rva % pointer_size as u64 != 0 {
                current_rva = (current_rva + pointer_size as u64 - 1) & !(pointer_size as u64 - 1);
            }
            for (_, functions) in &self.imports {
                current_rva += ((functions.len() as u64) + 1) * pointer_size as u64;
                // 包含终止符
            }

            // IAT（FirstThunk）按 pointer_size 字节对齐并分配 -> 这里就是 IAT 起始 RVA
            if current_rva % pointer_size as u64 != 0 {
                current_rva = (current_rva + pointer_size as u64 - 1) & !(pointer_size as u64 - 1);
            }
        }

        let iat_start_rva = current_rva;

        while i < code.len() {
            // x64: 修补多种指令的 [rip+disp32] 寻址
            if arch == Architecture::X86_64 && i + 2 < code.len() {
                let mut pos = i;
                let mut _rex_prefix = None;
                // 检查是否为 REX 前缀 (0x40 - 0x4F)
                if code[pos] >= 0x40 && code[pos] <= 0x4F {
                    _rex_prefix = Some(code[pos]);
                    pos += 1;
                }

                if pos + 1 < code.len() {
                    let mut opcode = code[pos] as u32;
                    let mut modrm_pos = pos + 1;

                    // 处理 2 字节指令 (0x0F 开头)
                    if opcode == 0x0F && pos + 2 < code.len() {
                        opcode = (opcode << 8) | (code[pos + 1] as u32);
                        modrm_pos = pos + 2;
                    }

                    if modrm_pos < code.len() {
                        let modrm = code[modrm_pos];

                        // 检查 ModR/M 是否为 [RIP + disp32] (mod=00, rm=101)
                        if (modrm & 0xC7) == 0x05 && modrm_pos + 4 <= code.len() {
                            let disp_offset = modrm_pos + 1;
                            let next_instr_offset = disp_offset + 4;

                            // 分支 1: 间接调用/跳转 (CALL/JMP [RIP+disp32]) -> 指向导入表 (IAT)
                            // CALL: FF /2, JMP: FF /4
                            if opcode == 0xFF && ((modrm >> 3) & 7 == 2 || (modrm >> 3) & 7 == 4) {
                                // 读取当前的 displacement 作为导入索引
                                let import_index = u32::from_le_bytes([
                                    code[disp_offset],
                                    code[disp_offset + 1],
                                    code[disp_offset + 2],
                                    code[disp_offset + 3],
                                ]) as usize;

                                // 计算实际的 IAT RVA
                                let mut current_iat_offset = 0;
                                let mut found_rva = None;
                                let mut flat_idx = 0;

                                'import_search: for (_, functions) in &self.imports {
                                    for _ in functions {
                                        if flat_idx == import_index {
                                            found_rva = Some(iat_start_rva + current_iat_offset);
                                            break 'import_search;
                                        }
                                        flat_idx += 1;
                                        current_iat_offset += pointer_size as u64;
                                    }
                                    current_iat_offset += pointer_size as u64; // 跳过 DLL 终止符
                                }

                                let target_rva =
                                    found_rva.unwrap_or(iat_start_rva + (import_index as u64 * pointer_size as u64));

                                let rip_rva: u64 = (code_section_rva as u64) + (next_instr_offset as u64);
                                let disp_i32 = (target_rva as i64 - rip_rva as i64) as i32;

                                code[disp_offset..disp_offset + 4].copy_from_slice(&disp_i32.to_le_bytes());

                                // 调试输出
                                tracing::trace!(
                                    "IMPORT 修补(RIP-relative): i={}, opcode={:04X}, rip_rva={:08X}, target_rva={:08X}, disp={:08X}, idx={}",
                                    i,
                                    opcode,
                                    rip_rva as u32,
                                    target_rva as u32,
                                    disp_i32 as u32,
                                    import_index
                                );

                                i = next_instr_offset;
                                continue;
                            }
                            // 分支 2: 数据访问指令 -> 指向数据段 (.data)
                            // 只要是 [RIP + disp32] 且不是上面处理过的导入调用，都视为数据访问
                            else {
                                // 特殊处理 lea 指令的重定位逻辑
                                // 检查是否为 LEA 指令 (0x8D)
                                if opcode == 0x8D {
                                    // 检查 REX 前缀是否包含 W 位 (0x08)
                                    // 对于 64 位 LEA，REX 应该是 0x48 (W=1) 或 0x4C (W=1, R=1) 等
                                    let is_64_lea = if let Some(rex) = _rex_prefix { (rex & 0x08) != 0 } else { false };

                                    if is_64_lea {
                                        // 读取当前 displacement 作为数据索引
                                        let data_index = u32::from_le_bytes([
                                            code[disp_offset],
                                            code[disp_offset + 1],
                                            code[disp_offset + 2],
                                            code[disp_offset + 3],
                                        ]) as usize;

                                        // 计算数据 RVA
                                        let data_rva = data_section_rva.unwrap_or(code_section_rva + 0x1000) as u64;
                                        let target_rva = data_rva + (data_index as u64);
                                        let rip_rva: u64 = (code_section_rva as u64) + (next_instr_offset as u64);
                                        let disp_i32 = (target_rva as i64 - rip_rva as i64) as i32;

                                        code[disp_offset..disp_offset + 4].copy_from_slice(&disp_i32.to_le_bytes());

                                        // 调试输出
                                        tracing::trace!(
                                            "LEA DATA 修补(RIP-relative): i={}, opcode={:04X}, rip_rva={:08X}, target_rva={:08X}, disp={:08X}",
                                            i,
                                            opcode,
                                            rip_rva as u32,
                                            target_rva as u32,
                                            disp_i32 as u32
                                        );
                                    }
                                    else {
                                        // 非 64 位 LEA (例如使用 32 位寄存器的 lea eax, [rip+disp32])
                                        // 读取当前 displacement 作为数据段内的偏移
                                        let current_disp = u32::from_le_bytes([
                                            code[disp_offset],
                                            code[disp_offset + 1],
                                            code[disp_offset + 2],
                                            code[disp_offset + 3],
                                        ]);

                                        let target_rva: u64 = (data_section_rva.unwrap_or(0) as u64) + (current_disp as u64);
                                        let rip_rva: u64 = (code_section_rva as u64) + (next_instr_offset as u64);
                                        let disp_i32 = (target_rva as i64 - rip_rva as i64) as i32;

                                        code[disp_offset..disp_offset + 4].copy_from_slice(&disp_i32.to_le_bytes());

                                        // 调试输出
                                        tracing::trace!(
                                            "LEA DATA 修补(RVA): i={}, opcode={:04X}, rip_rva={:08X}, target_rva={:08X}, disp={:08X}",
                                            i,
                                            opcode,
                                            rip_rva as u32,
                                            target_rva as u32,
                                            disp_i32 as u32
                                        );
                                    }
                                }
                                else {
                                    // 其他 [RIP + disp32] 指令 (如 mov, cmp 等)
                                    // 读取当前 displacement 作为数据段内的偏移
                                    let current_disp = u32::from_le_bytes([
                                        code[disp_offset],
                                        code[disp_offset + 1],
                                        code[disp_offset + 2],
                                        code[disp_offset + 3],
                                    ]);

                                    let target_rva: u64 = (data_section_rva.unwrap_or(0) as u64) + (current_disp as u64);
                                    let rip_rva: u64 = (code_section_rva as u64) + (next_instr_offset as u64);
                                    let disp_i32 = (target_rva as i64 - rip_rva as i64) as i32;

                                    code[disp_offset..disp_offset + 4].copy_from_slice(&disp_i32.to_le_bytes());

                                    // 调试输出
                                    tracing::trace!(
                                        "DATA 修补(RVA): i={}, opcode={:04X}, rip_rva={:08X}, target_rva={:08X}, disp={:08X}",
                                        i,
                                        opcode,
                                        rip_rva as u32,
                                        target_rva as u32,
                                        disp_i32 as u32
                                    );
                                }
                            }

                            i = next_instr_offset;
                            continue;
                        }
                    }
                }
            }

            // x86 (32-bit) 的处理逻辑保持不变
            if arch == Architecture::X86 && i + 1 < code.len() && code[i] == 0xFF && code[i + 1] == 0x15 && i + 5 < code.len() {
                // 读取当前的 displacement 作为导入索引
                let import_index = u32::from_le_bytes([code[i + 2], code[i + 3], code[i + 4], code[i + 5]]) as usize;

                // 计算实际的 IAT RVA
                let mut current_iat_offset = 0;
                let mut found_rva = None;
                let mut flat_idx = 0;

                'import_search_x86: for (_, functions) in &self.imports {
                    for _ in functions {
                        if flat_idx == import_index {
                            found_rva = Some(iat_start_rva + current_iat_offset);
                            break 'import_search_x86;
                        }
                        flat_idx += 1;
                        current_iat_offset += pointer_size as u64;
                    }
                    current_iat_offset += pointer_size as u64;
                }

                let target_rva = found_rva.unwrap_or(iat_start_rva + (import_index as u64 * pointer_size as u64));
                let target_va = image_base + target_rva;

                let disp: u32 = target_va as u32;
                let address_bytes = disp.to_le_bytes();
                code[i + 2..i + 6].copy_from_slice(&address_bytes);
                i += 6;
            }
            // 查找直接 CALL 指令 (0xE8) - 保留用于内部函数调用
            else if code[i] == 0xE8 && i + 4 < code.len() {
                // 读取相对偏移
                let rel_offset = i32::from_le_bytes([code[i + 1], code[i + 2], code[i + 3], code[i + 4]]);

                // 调试输出：由于目前 AOT 不使用内部标签，正常的直接 CALL 不应该在这里出现，除非是预留的占位符。
                // 如果偏移为 0，可能是一个需要修补的内部函数标签占位符。
                // 我们保留 rel_offset 的值，以便未来扩展。
                tracing::trace!("直接 CALL (E8): i={}, rel_offset={:08X}", i, rel_offset);

                i += 5; // 跳过整个直接 CALL 指令
            }
            // 查找 PUSH imm32 指令 (0x68) - 用于 push_label 或数据地址占位符
            else if code[i] == 0x68 && i + 4 < code.len() {
                // 读取立即数
                let imm = u32::from_le_bytes([code[i + 1], code[i + 2], code[i + 3], code[i + 4]]);

                if arch == Architecture::X86 {
                    // 如果这是一个 0 占位符，检查是否应该修补为 .data 开始地址
                    if imm == 0 {
                        // 检查前面是否有 push_imm8 + push_label 的模式
                        // push_imm(msg_len) 编译为 6a <len>（2字节）
                        // push_label("msg") 编译为 68 00 00 00 00（5字节）
                        let mut should_patch = false;

                        // 向前查找，看是否有 push imm8 <msg_len> 紧接着当前的 push 0 的模式
                        if i >= 2 {
                            // 确保有足够的空间向前查找
                            // 查找前面的 push imm8 指令 (6a)
                            let prev_push_pos = i - 2;
                            if prev_push_pos < code.len() && code[prev_push_pos] == 0x6a {
                                let prev_imm8 = code[prev_push_pos + 1] as u32;

                                // 检查这个立即数是否等于消息长度
                                let msg_len = if let Some(data) = &self.data {
                                    data.iter().position(|&b| b == 0).map(|p| p as u32).unwrap_or(0)
                                }
                                else {
                                    0
                                };

                                if prev_imm8 == msg_len {
                                    should_patch = true;
                                }
                            }
                        }

                        if should_patch {
                            // 数据段起始 VA
                            let data_section_va: u64 = image_base + (data_section_rva.unwrap_or(0) as u64);
                            let addr_u32 = data_section_va as u32;
                            code[i + 1..i + 5].copy_from_slice(&addr_u32.to_le_bytes());
                        }
                    }
                }

                i += 5; // 跳过整个 PUSH 指令
            }
            else {
                i += 1;
            }
        }
    }

    /// 生成 PE 文件字节数组
    pub fn generate(&mut self) -> Result<Vec<u8>, GaiaError> {
        // 构建节
        self.sections = self.build_sections(); // Populate sections first

        // 构建头部
        let header = self.build_header()?;

        // 构建导入表
        let mut import_table = ImportTable::new();
        for (dll_name, functions) in &self.imports {
            let entry = ImportEntry { dll_name: dll_name.clone(), functions: functions.clone() };
            import_table.entries.push(entry);
        }

        // 创建 PE 程序
        let program = PeProgram {
            header,
            sections: self.sections.clone(),
            imports: import_table,
            exports: crate::types::tables::ExportTable::new(),
        };

        // 写入到字节数组
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let mut writer = ExeWriter::new(cursor);
        writer.write_program(&program)?;

        Ok(buffer)
    }
}

impl Default for PeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
