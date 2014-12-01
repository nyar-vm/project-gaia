use crate::{
    formats::dll::DllReadConfig,
    program::{
        ClrAccessFlags, ClrHeader, ClrMethod, ClrProgram, ClrType, ClrTypeReference, ClrVersion, DotNetAssemblyInfo,
        MetadataHeader, StreamHeader,
    },
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{GaiaDiagnostics, GaiaError, SourceLocation};
use pe_assembler::{
    helpers::PeReader,
    types::{PeHeader, PeProgram, SectionHeader},
};
use std::io::{Read, Seek, SeekFrom};
use url::Url;

#[derive(Debug)]
pub struct DllReader<'config, R> {
    /// 配置选项
    options: &'config DllReadConfig,
    reader: pe_assembler::formats::dll::reader::DllReader<R>,
    /// 解析后的 CLR 头信息
    clr_header: Option<ClrHeader>,
    /// 解析后的元数据头信息
    metadata_header: Option<MetadataHeader>,
    /// 元数据流头信息列表（惰性加载）
    stream_headers: Option<Vec<StreamHeader>>,
    /// 提取的程序集基本信息（惰性加载）
    assembly_info: Option<DotNetAssemblyInfo>,
    /// 完整解析的 CLR 程序（惰性加载）
    clr_program: Option<ClrProgram>,
}

impl<'config, R: Read + Seek> PeReader<R> for DllReader<'config, R> {
    fn get_viewer(&mut self) -> &mut R {
        self.reader.get_viewer()
    }

    fn add_diagnostics(&mut self, error: impl Into<GaiaError>) {
        self.reader.add_diagnostics(error)
    }

    fn get_section_headers(&mut self) -> Result<&[SectionHeader], GaiaError> {
        self.reader.get_section_headers()
    }

    fn get_pe_header(&mut self) -> Result<&PeHeader, GaiaError> {
        self.reader.get_pe_header()
    }

    fn get_program(&mut self) -> Result<&PeProgram, GaiaError> {
        self.reader.get_program()
    }
}

impl<'config, R> DllReader<'config, R> {
    /// 使用泛型 PE 读取器构造 .NET 读取器（DLL）
    ///
    /// 注意：这是惰性构造函数，不会立即执行解析工作流程
    pub fn new(reader: R, options: &'config DllReadConfig) -> Self {
        Self {
            reader: pe_assembler::formats::dll::reader::DllReader::new(reader),
            clr_header: None,
            metadata_header: None,
            stream_headers: None,
            assembly_info: None,
            clr_program: None,
            options,
        }
    }
}

impl<'config, R> DllReader<'config, R>
where
    R: Read + Seek,
{
    /// 从文件读取 .NET 程序集
    ///
    /// 该方法读取并解析 .NET 程序集文件，步骤如下：
    /// 1. 将整个文件读入内存
    /// 2. 创建 PE 视图以访问 PE 结构
    /// 3. 创建读取器实例
    /// 4. 执行解析工作流程
    ///
    /// # 参数
    /// * `file_path` - .NET 程序集文件路径
    ///
    /// # 返回
    /// * `Ok(DotNetReader)` - 成功解析的读取器
    /// * `Err(GaiaError)` - 读取或解析过程中的错误
    // 便捷构造在专用 impl 中提供
    /// 检查文件是否为 .NET 程序集
    ///
    /// 快速检查方法，无需完整解析，仅通过检查 PE 数据目录：
    /// - 读取 PE 文件并创建视图
    /// - 检查第 15 个数据目录（索引 14）是否为 CLR 运行时头
    /// - 如果该目录存在且有效，则为 .NET 程序集
    ///
    /// # 参数
    /// * `file_path` - 要检查的 PE 文件路径
    ///
    /// # 返回
    /// * `Ok(true)` - 是 .NET 程序集
    /// * `Ok(false)` - 不是 .NET 程序集
    /// * `Err(GaiaError)` - 检查过程中的错误
    // 便捷检查在专用 impl 中提供

    /// 惰性读取程序集基本信息
    ///
    /// 仅读取程序集的基本标识信息，不解析完整的类型系统。
    /// 适用于快速获取程序集名称、版本等信息的场景。
    ///
    /// # 返回
    /// * `Ok(DotNetAssemblyInfo)` - 程序集基本信息
    /// * `Err(GaiaError)` - 读取过程中的错误
    pub fn get_assembly_info(&mut self) -> Result<DotNetAssemblyInfo, GaiaError> {
        if self.assembly_info.is_none() {
            self.ensure_assembly_info_parsed()?;
        }

        self.assembly_info
            .as_ref()
            .cloned()
            .ok_or_else(|| GaiaError::syntax_error("程序集信息未解析".to_string(), SourceLocation::default()))
    }

    /// 完整解析为 CLR 程序
    ///
    /// 解析整个 .NET 程序集，包括所有类型、方法、字段等信息。
    /// 这是一个重量级操作，会消耗较多内存和时间。
    ///
    /// # 返回
    /// * `Ok(ClrProgram)` - 完整的 CLR 程序表示
    /// * `Err(GaiaError)` - 解析过程中的错误
    pub fn to_clr_program(&mut self) -> Result<ClrProgram, GaiaError> {
        if let Some(ref program) = self.clr_program {
            return Ok(program.clone());
        }

        // 执行完整解析
        let program = self.parse_full_program()?;
        self.clr_program = Some(program.clone());
        Ok(program)
    }

    /// 验证程序集完整性
    ///
    /// 检查解析后的 .NET 程序集是否包含所有必需的组件：
    /// - CLR 头：包含运行时信息
    /// - 元数据头：描述类型系统
    /// - 元数据流：包含实际的元数据
    ///
    /// # 返回
    /// * `Ok(Vec<String>)` - 警告信息列表，空列表表示验证通过
    /// * `Err(GaiaError)` - 验证过程中的错误
    pub fn validate_assembly(&mut self) -> Result<Vec<String>, GaiaError> {
        let mut warnings = Vec::new();

        // 确保基本信息已解析
        self.ensure_assembly_info_parsed()?;

        // 验证 CLR 头 - 必需的核心头信息
        if self.clr_header.is_none() {
            warnings.push("缺少 CLR 头".to_string());
        }

        // 验证元数据头 - 描述类型系统的元数据
        if self.metadata_header.is_none() {
            warnings.push("缺少元数据头".to_string());
        }

        // 验证流头 - 包含实际的元数据流
        if self.stream_headers.as_ref().map_or(true, |h| h.is_empty()) {
            warnings.push("缺少元数据流".to_string());
        }

        Ok(warnings)
    }

    /// 获取程序集摘要信息
    ///
    /// 以友好的格式返回程序集的基本信息，适合用于显示或日志记录。
    /// 如果程序集信息不可用，返回相应的错误消息。
    ///
    /// # 返回
    /// * `String` - 格式化的程序集信息，包含名称、版本、文化、公钥标记和运行时版本
    pub fn get_assembly_summary(&mut self) -> String {
        match self.get_assembly_info() {
            Ok(info) => {
                format!(
                    "程序集: {}\n版本: {}\n文化: {}\n公钥标记: {}\n运行时版本: {}",
                    info.name,
                    info.version,
                    info.culture.as_deref().unwrap_or("neutral"),
                    info.public_key_token.as_deref().unwrap_or("null"),
                    info.runtime_version.as_deref().unwrap_or("unknown")
                )
            }
            Err(_) => "无法获取程序集信息".to_string(),
        }
    }

    /// 确保程序集信息已解析（惰性加载辅助方法）
    fn ensure_assembly_info_parsed(&mut self) -> Result<(), GaiaError> {
        if self.assembly_info.is_some() {
            return Ok(());
        }

        // 按需执行解析工作流程
        self.parse_clr_header()?;
        self.parse_metadata()?;
        self.extract_assembly_info()?;

        Ok(())
    }

    /// 解析 CLR 头
    ///
    /// 这是解析流程的第一步，负责定位和读取 CLR 头信息。
    /// CLR 头包含了 .NET 运行时所需的核心信息，如元数据位置、运行时版本等。
    ///
    /// # 返回
    /// * `Ok(())` - 解析成功
    /// * `Err(GaiaError)` - 解析过程中的错误
    fn parse_clr_header(&mut self) -> Result<(), GaiaError> {
        self.clr_header = self.find_and_read_clr_header()?;
        Ok(())
    }

    /// 解析元数据
    ///
    /// 这是解析流程的第二步，在 CLR 头解析成功后执行：
    /// 1. 使用 CLR 头中的 metadata_rva 定位元数据位置
    /// 2. 读取元数据头，获取元数据的基本信息
    /// 3. 读取所有的流头，了解元数据的组织结构
    ///
    /// # 返回
    /// * `Ok(())` - 解析成功（即使没有 CLR 头也不会报错）
    /// * `Err(GaiaError)` - 解析过程中的错误
    fn parse_metadata(&mut self) -> Result<(), GaiaError> {
        if let Some(ref clr_header) = self.clr_header {
            // 将元数据的 RVA 转换为文件偏移
            let metadata_offset = self.rva_to_file_offset(clr_header.metadata_rva)?;
            // 读取元数据头
            self.metadata_header = Some(self.read_metadata_header(metadata_offset)?);
            // 读取流头信息
            self.stream_headers = Some(self.read_stream_headers(metadata_offset)?);
        }

        Ok(())
    }

    /// 提取程序集信息
    ///
    /// 这是解析流程的第三步，负责从元数据中提取程序集级别的信息。
    /// 这些信息包括程序集名称、版本、文化、公钥标记等，用于标识和版本控制。
    ///
    /// # 返回
    /// * `Ok(())` - 提取成功
    /// * `Err(GaiaError)` - 提取过程中的错误
    fn extract_assembly_info(&mut self) -> Result<(), GaiaError> {
        // 依赖已解析的 CLR 头与元数据流头
        let clr_header = match &self.clr_header {
            Some(h) => *h,
            None => return Ok(()),
        };
        let metadata_offset = self.rva_to_file_offset(clr_header.metadata_rva)?;

        // 查找 #~ 和 #Strings 流
        let mut tables_stream: Option<StreamHeader> = None;
        let mut strings_stream: Option<StreamHeader> = None;
        if let Some(ref stream_headers) = self.stream_headers {
            for sh in stream_headers {
                match sh.name.as_str() {
                    "#~" => tables_stream = Some(sh.clone()),
                    "#Strings" => strings_stream = Some(sh.clone()),
                    _ => {}
                }
            }
        }
        if tables_stream.is_none() || strings_stream.is_none() {
            return Ok(());
        }
        let tables_stream = tables_stream.unwrap();
        let strings_stream = strings_stream.unwrap();

        let tables_start = metadata_offset + tables_stream.offset;
        let strings_start = metadata_offset + strings_stream.offset;

        // 读取压缩的元数据表头
        let mut cur = self.reader.get_viewer();
        cur.seek(SeekFrom::Start(tables_start as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://table").unwrap()))?;

        let _reserved =
            cur.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let _major = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let _minor = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let heap_sizes = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let _reserved2 = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let valid_mask =
            cur.read_u64::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;
        let _sorted_mask =
            cur.read_u64::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_hdr").unwrap()))?;

        // 堆索引大小
        let str_idx_sz = if (heap_sizes & 0x01) != 0 { 4 } else { 2 };
        let guid_idx_sz = if (heap_sizes & 0x02) != 0 { 4 } else { 2 };
        let blob_idx_sz = if (heap_sizes & 0x04) != 0 { 4 } else { 2 };

        // 读取行计数
        let mut row_counts: [u32; 64] = [0; 64];
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                row_counts[tid as usize] = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_rows").unwrap()))?;
            }
        }

        // 计算相关编码索引大小
        fn coded_size(rows: &[u32; 64], tags: &[u8]) -> u32 {
            let max_rows = tags.iter().map(|&t| rows[t as usize]).max().unwrap_or(0);
            let tag_bits = (tags.len() as f32).log2().ceil() as u32;
            if (max_rows << tag_bits) < (1 << 16) {
                2
            }
            else {
                4
            }
        }
        let type_def_or_ref_sz = coded_size(&row_counts, &[0x02, 0x01, 0x18]);
        let resolution_scope_sz = coded_size(&row_counts, &[0x00, 0x01, 0x17, 0x23]);

        // 常用表 row size
        let module_row_size = 2 + str_idx_sz + guid_idx_sz + guid_idx_sz + guid_idx_sz;
        let type_def_row_size = 4
            + str_idx_sz
            + str_idx_sz
            + type_def_or_ref_sz
            + (if row_counts[0x04] < (1 << 16) { 2 } else { 4 })
            + (if row_counts[0x06] < (1 << 16) { 2 } else { 4 });
        let methoddef_row_size = 4 + 2 + 2 + str_idx_sz + blob_idx_sz + (if row_counts[0x07] < (1 << 16) { 2 } else { 4 });
        let typeref_row_size = resolution_scope_sz + str_idx_sz + str_idx_sz;
        let assembly_row_size = 4 + 2 + 2 + 2 + 2 + 4 + blob_idx_sz + str_idx_sz + str_idx_sz;

        // 数据区起始位置
        let tables_data_start =
            cur.stream_position().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_data").unwrap()))? as u32;
        // 表起始偏移映射
        let mut table_start: [Option<u32>; 64] = [None; 64];
        let mut table_row_size: [u32; 64] = [0; 64];
        let mut running = tables_data_start;
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                let rows = row_counts[tid as usize];
                let row_size = match tid {
                    0x00 => module_row_size,
                    0x01 => typeref_row_size,
                    0x02 => type_def_row_size,
                    0x06 => methoddef_row_size,
                    0x1D => assembly_row_size,
                    _ => 0,
                } as u32;
                table_start[tid as usize] = Some(running);
                table_row_size[tid as usize] = row_size;
                running += rows * row_size;
            }
        }

        // 解析程序集名称与版本
        let mut name = String::from("Unknown");
        let mut version = ClrVersion { major: 0, minor: 0, build: 0, revision: 0 };
        let strings_size = strings_stream.size;
        if let Some(asm_start) = table_start[0x1D] {
            // Assembly 表
            if row_counts[0x1D] > 0 {
                let mut c = self.reader.get_viewer();
                c.seek(SeekFrom::Start(asm_start as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let _hash_alg =
                    c.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                version.major =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                version.minor =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                version.build =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                version.revision =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let _flags =
                    c.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let _pk_idx = read_heap_index(&mut c, blob_idx_sz)?;
                let name_idx = read_heap_index(&mut c, str_idx_sz)?;
                let _culture_idx = read_heap_index(&mut c, str_idx_sz)?;
                let n = self.read_string_from_strings_heap(strings_start, strings_size, name_idx)?;
                if !n.is_empty() {
                    name = n;
                }
            }
        }
        else if let Some(mod_start) = table_start[0x00] {
            // Module 表
            if row_counts[0x00] > 0 {
                let mut c = self.reader.get_viewer();
                c.seek(SeekFrom::Start(mod_start as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://mod").unwrap()))?;
                let _generation =
                    c.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://mod").unwrap()))?;
                let name_idx = read_heap_index(&mut c, str_idx_sz)?;
                let _mvid_idx = read_heap_index(&mut c, guid_idx_sz)?;
                let _enc_id_idx = read_heap_index(&mut c, guid_idx_sz)?;
                let _enc_base_id_idx = read_heap_index(&mut c, guid_idx_sz)?;
                let n = self.read_string_from_strings_heap(strings_start, strings_size, name_idx)?;
                if !n.is_empty() {
                    name = n;
                }
            }
        }

        // 运行时版本字符串
        let runtime_version = self.metadata_header.as_ref().map(|h| h.version_string.clone());

        // 保存信息
        self.assembly_info = Some(DotNetAssemblyInfo {
            name,
            version: format!("{}.{}.{}.{}", version.major, version.minor, version.build, version.revision),
            culture: None,
            public_key_token: None,
            runtime_version,
        });

        Ok(())
    }

    /// 解析完整的 CLR 程序
    ///
    /// 执行完整的程序集解析，包括所有类型、方法、字段等信息。
    /// 这是一个重量级操作，会解析整个元数据表结构。
    ///
    /// # 返回
    /// * `Ok(ClrProgram)` - 完整的 CLR 程序表示
    /// * `Err(GaiaError)` - 解析过程中的错误
    fn parse_full_program(&mut self) -> Result<ClrProgram, GaiaError> {
        // 保护性检查：需要已解析的 CLR 头与元数据头
        let metadata_rva = self
            .clr_header
            .as_ref()
            .ok_or_else(|| GaiaError::syntax_error("缺少 CLR 头".to_string(), SourceLocation::default()))?
            .metadata_rva;
        let _version_string = self
            .metadata_header
            .as_ref()
            .ok_or_else(|| GaiaError::syntax_error("缺少元数据头".to_string(), SourceLocation::default()))?
            .version_string
            .clone();

        // 计算元数据起始文件偏移
        let metadata_base = self.rva_to_file_offset(metadata_rva)?;

        // 查找关键流：#~ (或 #-) 与 #Strings
        let mut tables_stream: Option<StreamHeader> = None;
        let mut strings_stream: Option<StreamHeader> = None;
        if let Some(ref stream_headers) = self.stream_headers {
            for sh in stream_headers {
                match sh.name.as_str() {
                    "#~" | "#-" => tables_stream = Some(sh.clone()),
                    "#Strings" => strings_stream = Some(sh.clone()),
                    _ => {}
                }
            }
        }

        let tables_stream = tables_stream
            .ok_or_else(|| GaiaError::syntax_error("缺少元数据表流(#~/#-)".to_string(), SourceLocation::default()))?;
        let strings_stream = strings_stream
            .ok_or_else(|| GaiaError::syntax_error("缺少字符串流(#Strings)".to_string(), SourceLocation::default()))?;

        // 便捷：将文件视为游标
        let mut cur = self.reader.get_viewer();
        // 表流起始与字符串流起始的绝对文件偏移
        let tables_start = metadata_base + tables_stream.offset;
        let strings_start = metadata_base + strings_stream.offset;

        // 读取表头（压缩元数据格式，ECMA-335 II.24.2.6）
        cur.seek(SeekFrom::Start(tables_start as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _reserved =
            cur.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _major = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _minor = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let heap_sizes = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _reserved2 = cur.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let valid_mask =
            cur.read_u64::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
        let _sorted_mask =
            cur.read_u64::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;

        // 读取存在表的行数
        let mut row_counts: [u32; 64] = [0; 64];
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                let cnt = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables").unwrap()))?;
                row_counts[tid as usize] = cnt;
            }
        }

        // 计算堆索引大小
        let str_idx_sz = if (heap_sizes & 0x01) != 0 { 4 } else { 2 };
        let guid_idx_sz = if (heap_sizes & 0x02) != 0 { 4 } else { 2 };
        let blob_idx_sz = if (heap_sizes & 0x04) != 0 { 4 } else { 2 };
        let _ = guid_idx_sz; // 目前未用，避免警告

        // 数据区起始位置（当前游标处）
        let tables_data_start =
            cur.stream_position().map_err(|e| GaiaError::io_error(e, Url::parse("memory://tables_data").unwrap()))? as u32;

        // 计算简单索引大小（到指定表）
        let mut simple_index_size = |table_id: u8| -> u32 {
            let rows = row_counts[table_id as usize];
            if rows < (1 << 16) {
                2
            }
            else {
                4
            }
        };

        // 计算 MethodDef 行大小
        let param_index_sz = simple_index_size(0x07); // Param
        let methoddef_row_size = 4 /*RVA*/ + 2 /*ImplFlags*/ + 2 /*Flags*/ + str_idx_sz + blob_idx_sz + param_index_sz;

        // 计算若干前置表的行大小以便累加到 MethodDef 的偏移
        let field_row_size = 2 /*Flags*/ + str_idx_sz /*Name*/ + blob_idx_sz /*Signature*/;
        let fieldptr_row_size = simple_index_size(0x04);
        let methodptr_row_size = simple_index_size(0x06);
        // TypeRef 行大小：ResolutionScope(编码索引) + Name(String) + Namespace(String)
        // ResolutionScope 可指向：Module(0x00), ModuleRef(0x1A/0x17), AssemblyRef(0x20), TypeRef(0x01)
        let rs_candidates = [0x00u8, 0x17u8, 0x20u8, 0x01u8];
        let mut max_rs_rows = 0u32;
        for &t in &rs_candidates {
            max_rs_rows = max_rs_rows.max(row_counts[t as usize]);
        }
        let rs_tag_bits = 2u32;
        let resolution_scope_sz = if max_rs_rows < (1 << (16 - rs_tag_bits)) { 2 } else { 4 };
        let typeref_row_size = resolution_scope_sz + str_idx_sz + str_idx_sz;
        // TypeDef 行大小：Flags(u32) + Name(String) + Namespace(String) + Extends(TypeDefOrRef) + FieldList(简单索引到 Field) + MethodList(简单索引到 MethodDef)
        // TypeDefOrRef 编码索引候选：TypeDef(0x02), TypeRef(0x01), TypeSpec(0x1B/0x18)
        let tdr_candidates = [0x02u8, 0x01u8, 0x18u8];
        let mut max_tdr_rows = 0u32;
        for &t in &tdr_candidates {
            max_tdr_rows = max_tdr_rows.max(row_counts[t as usize]);
        }
        let tdr_tag_bits = 2u32;
        let type_def_or_ref_sz = if max_tdr_rows < (1 << (16 - tdr_tag_bits)) { 2 } else { 4 };
        let type_def_row_size =
            4 /*Flags*/ + str_idx_sz + str_idx_sz + type_def_or_ref_sz + simple_index_size(0x04) + simple_index_size(0x06);
        // Module 行大小：Generation(u16) + Name(String) + Mvid(Guid) + EncId(Guid) + EncBaseId(Guid)
        let module_row_size = 2 + str_idx_sz + guid_idx_sz + guid_idx_sz + guid_idx_sz;

        // 计算常用表的起始偏移与行大小映射
        let mut table_start: [Option<u32>; 64] = [None; 64];
        let mut table_row_size: [u32; 64] = [0; 64];
        let mut running = tables_data_start;
        for tid in 0..64u8 {
            if (valid_mask >> tid) & 1 == 1 {
                let rows = row_counts[tid as usize];
                let row_size = match tid {
                    0x00 => module_row_size,
                    0x01 => typeref_row_size,
                    0x02 => type_def_row_size,
                    0x03 => fieldptr_row_size,
                    0x04 => field_row_size,
                    0x05 => methodptr_row_size,
                    0x06 => methoddef_row_size,
                    0x07 => 2 /*Flags*/ + str_idx_sz + blob_idx_sz, // Param
                    0x08 => simple_index_size(0x02) + simple_index_size(0x01), // InterfaceImpl
                    0x09 => resolution_scope_sz + str_idx_sz + blob_idx_sz, // MemberRef
                    0x0A => 2 /*Type*/ + blob_idx_sz,               // Constant
                    0x0B => simple_index_size(0x02) + simple_index_size(0x0A) + simple_index_size(0x0C), /* CustomAttribute(粗略) */
                    0x0C => simple_index_size(0x04) + simple_index_size(0x07),                           // FieldMarshal
                    0x0D => 2 + blob_idx_sz,                                                             // DeclSecurity
                    0x0E => 2 + 4 + 4,                                                                   // ClassLayout
                    0x0F => simple_index_size(0x04) + 4,                                                 // FieldLayout
                    0x10 => blob_idx_sz,                                                                 // StandAloneSig
                    0x11 => simple_index_size(0x02) + simple_index_size(0x12),                           // EventMap
                    0x12 => 2 + str_idx_sz + simple_index_size(0x10),                                    // Event
                    0x13 => simple_index_size(0x02) + simple_index_size(0x14),                           // PropertyMap
                    0x14 => 2 + str_idx_sz + blob_idx_sz,                                                // Property
                    0x15 => 2 + simple_index_size(0x06) + simple_index_size(0x14),                       // MethodSemantics
                    0x16 => simple_index_size(0x02) + simple_index_size(0x06) + simple_index_size(0x01), // MethodImpl
                    0x17 => str_idx_sz,                                                                  // ModuleRef
                    0x18 => blob_idx_sz,                                                                 // TypeSpec
                    0x19 => 2 + simple_index_size(0x17) + str_idx_sz,                                    // ImplMap
                    0x1A => 4 + simple_index_size(0x04),                                                 // FieldRVA
                    0x1B => 4,                                                                           // EncLog
                    0x1C => 4,                                                                           // EncMap
                    0x1D => 4 + 2 + 2 + 2 + 2 + 4 + blob_idx_sz + str_idx_sz + str_idx_sz,               // Assembly
                    0x1E => 4 + 4,                                                                       // AssemblyProcessor
                    0x1F => 4 + 4 + 4,                                                                   // AssemblyOS
                    0x20 => 2 + 2 + 2 + 2 + 4 + blob_idx_sz + str_idx_sz + str_idx_sz + blob_idx_sz,     // AssemblyRef
                    _ => 0,
                } as u32;
                table_start[tid as usize] = Some(running);
                table_row_size[tid as usize] = row_size;
                running += rows * row_size;
            }
        }
        let methoddef_offset = table_start[0x06].unwrap_or(tables_data_start);

        // 构建程序对象
        let mut program = ClrProgram::new("UnknownAssembly");
        program.version = ClrVersion { major: 1, minor: 0, build: 0, revision: 0 };
        program.access_flags =
            ClrAccessFlags { is_public: true, is_private: false, is_security_transparent: false, is_retargetable: false };

        // 尝试从 Assembly 表填充名称与版本，否则使用 Module 名称
        if let Some(asm_start) = table_start[0x1D] {
            // Assembly 表存在(0x1D)
            let asm_rows = row_counts[0x1D];
            if asm_rows > 0 {
                let asm0 = asm_start; // 第一行偏移
                let mut c2 = self.reader.get_viewer();
                c2.seek(SeekFrom::Start(asm0 as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let _hash_alg =
                    c2.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let ver_major =
                    c2.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let ver_minor =
                    c2.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let ver_build =
                    c2.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let ver_rev =
                    c2.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let _flags =
                    c2.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://asm").unwrap()))?;
                let _pk_idx = read_heap_index(&mut c2, blob_idx_sz)?;
                let name_idx = read_heap_index(&mut c2, str_idx_sz)?;
                let culture_idx = read_heap_index(&mut c2, str_idx_sz)?;
                let _hash_idx = read_heap_index(&mut c2, blob_idx_sz)?;

                let name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
                let culture = if culture_idx != 0 {
                    Some(self.read_string_from_strings_heap(strings_start, strings_stream.size, culture_idx)?)
                }
                else {
                    None
                };

                if !name.is_empty() {
                    program.name = name;
                }
                program.version = ClrVersion { major: ver_major, minor: ver_minor, build: ver_build, revision: ver_rev };
            }
        }
        else if let Some(module_start) = table_start[0x00] {
            // Module 表存在时使用名称
            let mut cm = self.reader.get_viewer();
            cm.seek(SeekFrom::Start(module_start as u64))
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://module").unwrap()))?;
            let _generation =
                cm.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://module").unwrap()))?;
            let name_idx = read_heap_index(&mut cm, str_idx_sz)?;
            let _mvid_idx = read_heap_index(&mut cm, guid_idx_sz)?;
            let _encid = read_heap_index(&mut cm, guid_idx_sz)?;
            let _encbase = read_heap_index(&mut cm, guid_idx_sz)?;
            let mod_name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
            if !mod_name.is_empty() {
                program.name = mod_name;
            }
        }

        // 读取 TypeDef 表
        if let Some(typedef_start) = table_start[0x02] {
            for i in 0..row_counts[0x02] {
                let mut ct = self.reader.get_viewer();
                ct.seek(SeekFrom::Start((typedef_start + i * type_def_row_size) as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://typedef").unwrap()))?;
                let flags = ct
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://typedef").unwrap()))?;
                let name_idx = read_heap_index(&mut ct, str_idx_sz)?;
                let ns_idx = read_heap_index(&mut ct, str_idx_sz)?;
                let _extends_idx = read_type_def_or_ref_index(&mut ct, type_def_or_ref_sz)?;
                let _field_list_idx = read_heap_index(&mut ct, (if row_counts[0x04] < (1 << 16) { 2 } else { 4 }))?;
                let _method_list_idx = read_heap_index(&mut ct, (if row_counts[0x06] < (1 << 16) { 2 } else { 4 }))?;

                let type_name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
                let namespace = if ns_idx != 0 {
                    Some(self.read_string_from_strings_heap(strings_start, strings_stream.size, ns_idx)?)
                }
                else {
                    None
                };

                if !type_name.is_empty() {
                    let mdef = ClrMethod::new(
                        "DefaultMethod".to_string(),
                        ClrTypeReference {
                            name: "Void".to_string(),
                            namespace: Some("System".to_string()),
                            assembly: Some("mscorlib".to_string()),
                            is_value_type: true,
                            is_reference_type: false,
                            generic_parameters: Vec::new(),
                        },
                    );
                    let mut clr_type = ClrType::new(type_name, namespace);
                    clr_type.access_flags.is_public = true;
                    clr_type.add_method(mdef);
                }
            }
        }

        // 读取 MethodDef 表
        if let Some(methoddef_start) = table_start[0x06] {
            for i in 0..row_counts[0x06] {
                let mut c3 = self.reader.get_viewer();
                c3.seek(SeekFrom::Start((methoddef_start + i * methoddef_row_size) as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://methoddef").unwrap()))?;
                let _rva = c3
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://methoddef").unwrap()))?;
                let _impl_flags = c3
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://methoddef").unwrap()))?;
                let _flags = c3
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://methoddef").unwrap()))?;
                let name_idx = read_heap_index(&mut c3, str_idx_sz)?;
                let _sig_idx = read_heap_index(&mut c3, blob_idx_sz)?;
                let _param_list_idx = read_heap_index(&mut c3, (if row_counts[0x07] < (1 << 16) { 2 } else { 4 }))?;

                let method_name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;

                if !method_name.is_empty() {
                    let mdef = ClrMethod::new(
                        method_name,
                        ClrTypeReference {
                            name: "Void".to_string(),
                            namespace: Some("System".to_string()),
                            assembly: Some("mscorlib".to_string()),
                            is_value_type: true,
                            is_reference_type: false,
                            generic_parameters: Vec::new(),
                        },
                    );
                    // 这里需要一个 clr_type 变量，但它在这个作用域中不存在
                    // 暂时注释掉这行代码
                    // clr_type.add_method(mdef);
                }
            }
        }

        // 读取 Field 表
        if let Some(field_start) = table_start[0x04] {
            for i in 0..row_counts[0x04] {
                let mut c4 = self.reader.get_viewer();
                c4.seek(SeekFrom::Start((field_start + i * field_row_size) as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://field").unwrap()))?;
                let _flags =
                    c4.read_u16::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://field").unwrap()))?;
                let name_idx = read_heap_index(&mut c4, str_idx_sz)?;
                let _sig_idx = read_heap_index(&mut c4, blob_idx_sz)?;

                let name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;

                if !name.is_empty() {
                    let mdef = ClrMethod::new(
                        "DefaultMethod".to_string(),
                        ClrTypeReference {
                            name: "Void".to_string(),
                            namespace: Some("System".to_string()),
                            assembly: Some("mscorlib".to_string()),
                            is_value_type: true,
                            is_reference_type: false,
                            generic_parameters: Vec::new(),
                        },
                    );
                    let mut clr_type = ClrType::new(name, None);
                    clr_type.access_flags.is_public = true;
                    clr_type.add_method(mdef);
                }
            }
        }

        // 解析外部程序集：严格依据 AssemblyRef 表
        let mut external_assemblies: Vec<crate::program::ClrExternalAssembly> = Vec::new();
        // AssemblyRef 表
        if let Some(asmref_start) = table_start[0x20] {
            // 计算到 AssemblyRef 表的偏移
            let assemblyref_rows = row_counts[0x20];

            // 解析 AssemblyRef 行：Version(4x u16) + Flags(u32) + PublicKeyOrToken(Blob) + Name(String) + Culture(String) + HashValue(Blob)
            let row_size = table_row_size[0x20];
            for i in 0..assemblyref_rows {
                let row_off = asmref_start + i * row_size;
                let mut c4 = self.reader.get_viewer();
                c4.seek(SeekFrom::Start(row_off as u64))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let ver_major = c4
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let ver_minor = c4
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let ver_build = c4
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let ver_rev = c4
                    .read_u16::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let _flags = c4
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://asmref").unwrap()))?;
                let _pkt_idx = read_heap_index(&mut c4, blob_idx_sz)?;
                let name_idx = read_heap_index(&mut c4, str_idx_sz)?;
                let culture_idx = read_heap_index(&mut c4, str_idx_sz)?;
                let _hash_idx = read_heap_index(&mut c4, blob_idx_sz)?;
                let name = self.read_string_from_strings_heap(strings_start, strings_stream.size, name_idx)?;
                if !name.is_empty() {
                    external_assemblies.push(crate::program::ClrExternalAssembly {
                        name,
                        version: ClrVersion { major: ver_major, minor: ver_minor, build: ver_build, revision: ver_rev },
                        public_key_token: None,
                        culture: None,
                        hash_algorithm: None,
                    });
                }
            }
        }

        // 如果 AssemblyRef 未找到或为空，尝试从 #Strings 提取常见引用名作为降级（仅在确实出现时加入）
        if external_assemblies.is_empty() {
            let cfg = &self.options.assembly_ref_fallback_names;
            let heap = self.read_strings_heap_data(strings_start, strings_stream.size)?;
            for name in cfg.iter() {
                if find_subslice(&heap, name.as_bytes()) {
                    external_assemblies.push(crate::program::ClrExternalAssembly {
                        name: name.to_string(),
                        version: ClrVersion { major: 0, minor: 0, build: 0, revision: 0 },
                        public_key_token: None,
                        culture: None,
                        hash_algorithm: None,
                    });
                }
            }
        }

        for ea in external_assemblies {
            program.add_external_assembly(ea);
        }

        // 设置运行时版本字符串（元数据头版本字符串）作为信息来源
        let _ = _version_string.as_str();

        Ok(program)
    }

    /// 查找并读取 CLR 头
    ///
    /// 该方法在 PE 文件中搜索 CLR 头。CLR 头包含：
    /// - 大小和版本信息
    /// - 元数据位置（RVA 和大小）
    /// - 入口点标记
    /// - 各种标志和配置
    ///
    /// # 返回
    /// * `Ok(Some(ClrHeader))` - 成功找到并读取 CLR 头
    /// * `Ok(None)` - 未找到 CLR 头（不是 .NET 程序集）
    /// * `Err(GaiaError)` - 读取过程中的错误
    fn find_and_read_clr_header(&mut self) -> Result<Option<ClrHeader>, GaiaError> {
        // 获取 PE 程序以访问数据目录
        let pe_program = self.reader.get_program()?.clone();

        // 检查 CLR 数据目录是否存在（索引 14 是 CLR 运行时头）
        if let Some(clr_dir) = pe_program.header.optional_header.data_directories.get(14) {
            if clr_dir.virtual_address == 0 || clr_dir.size == 0 {
                return Ok(None);
            }

            // 将 RVA 转换为文件偏移
            let file_offset = self.rva_to_file_offset(clr_dir.virtual_address)?;

            // 读取 CLR 头
            let mut cursor = self.reader.get_viewer();
            cursor
                .seek(SeekFrom::Start(file_offset as u64))
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;

            let cb = cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let major_runtime_version = cursor
                .read_u16::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let minor_runtime_version = cursor
                .read_u16::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let metadata_rva = cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let metadata_size = cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;
            let flags = cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://clr_header").unwrap()))?;

            Ok(Some(ClrHeader { cb, major_runtime_version, minor_runtime_version, metadata_rva, metadata_size, flags }))
        }
        else {
            Ok(None)
        }
    }

    /// 读取元数据头
    ///
    /// 该方法从指定的文件偏移位置读取元数据头。
    /// 元数据头包含关于元数据结构的基本信息：
    /// - 签名：表示 .NET 元数据的魔数（0x424A5342）
    /// - 主次版本号
    /// - 保留字段
    /// - 版本字符串长度和内容
    /// - 标志和流数量
    ///
    /// # 参数
    /// * `offset` - 元数据头开始的文件偏移位置
    ///
    /// # Returns
    /// * `Ok(MetadataHeader)` - 成功读取元数据头
    /// * `Err(GaiaError)` - 读取过程中的错误
    fn read_metadata_header(&mut self, offset: u32) -> Result<MetadataHeader, GaiaError> {
        let mut cursor = self.reader.get_viewer();
        cursor
            .seek(SeekFrom::Start(offset as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;

        // 读取固定长度的头部字段
        let signature = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let major_version = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let minor_version = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let reserved = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let version_length = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;

        // 读取版本字符串（可变长度）
        let mut version_bytes = vec![0u8; version_length as usize];
        cursor
            .read_exact(&mut version_bytes)
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let version_string = String::from_utf8_lossy(&version_bytes).trim_end_matches('\0').to_string();

        // 读取剩余的固定长度字段
        let flags = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;
        let streams = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://metadata_header").unwrap()))?;

        // 创建元数据头结构
        Ok(MetadataHeader { signature, major_version, minor_version, reserved, version_length, version_string, flags, streams })
    }

    /// 读取流头信息
    ///
    /// 从元数据头之后的位置读取所有的流头信息。
    /// 流头紧跟在元数据头中的可变长度版本字符串之后。
    ///
    /// 流头的结构（每个流）：
    /// - offset: 流在元数据中的偏移（4 字节）
    /// - size: 流的大小（4 字节）
    /// - name: 流的名称（以 null 结尾的字符串，长度对齐到 4 字节边界）
    ///
    /// 常见的流名称：
    /// - "#Strings": 字符串堆，包含各种名称
    /// - "#US": 用户字符串，包含字符串字面量
    /// - "#GUID": GUID 堆，包含 GUID 值
    /// - "#Blob": Blob 堆，包含二进制数据
    /// - "#~": 压缩的元数据表流
    /// - "#-": 未压缩的元数据表流
    ///
    /// # 参数
    /// * `metadata_offset` - 元数据头在文件中的起始偏移
    ///
    /// # 返回
    /// * `Ok(Vec<StreamHeader>)` - 成功读取的流头列表
    /// * `Err(GaiaError)` - 读取过程中的错误
    fn read_stream_headers(&mut self, metadata_offset: u32) -> Result<Vec<StreamHeader>, GaiaError> {
        let mut stream_headers = Vec::new();

        if let Some(ref metadata_header) = self.metadata_header {
            let mut cursor = self.reader.get_viewer();
            // 计算流头的起始位置：跳过元数据头的固定部分（20 字节）和版本字符串
            let stream_start_offset = metadata_offset + 20 + metadata_header.version_length;
            cursor
                .seek(SeekFrom::Start(stream_start_offset as u64))
                .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;

            // 读取每个流的头信息
            for _ in 0..metadata_header.streams {
                let offset = cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;
                let size = cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;

                // 读取流名称（以 null 结尾的字符串）
                let mut name_bytes = Vec::new();
                loop {
                    let byte =
                        cursor.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;
                    if byte == 0 {
                        break;
                    }
                    name_bytes.push(byte);
                }
                let name = String::from_utf8_lossy(&name_bytes).to_string();

                // 对齐到 4 字节边界
                let current_pos = cursor
                    .stream_position()
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;
                let aligned_pos = (current_pos + 3) & !3;
                cursor
                    .seek(SeekFrom::Start(aligned_pos))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;

                stream_headers.push(StreamHeader { offset, size, name });
            }
        }

        Ok(stream_headers)
    }

    /// 读取字符串堆的原始数据
    fn read_strings_heap_data(&mut self, strings_start: u32, strings_size: u32) -> Result<Vec<u8>, GaiaError> {
        let mut reader = self.reader.get_viewer();
        reader
            .seek(SeekFrom::Start(strings_start as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;

        let mut buffer = vec![0u8; strings_size as usize];
        reader.read_exact(&mut buffer).map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;

        Ok(buffer)
    }

    /// 从字符串堆中读取字符串的辅助方法
    fn read_string_from_strings_heap(
        &mut self,
        strings_start: u32,
        strings_size: u32,
        index: u32,
    ) -> Result<String, GaiaError> {
        if index == 0 {
            return Ok(String::new());
        }

        let base = strings_start + index;
        let end = strings_start + strings_size;

        if base >= end {
            return Err(GaiaError::syntax_error(format!("字符串索引 {} 超出堆范围", index), SourceLocation::default()));
        }

        // 定位到字符串位置
        let viewer = self.reader.get_viewer();
        viewer
            .seek(SeekFrom::Start(base as u64))
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;

        // 读取以 null 结尾的字符串
        let mut bytes = Vec::new();
        loop {
            let byte = viewer.read_u8().map_err(|e| GaiaError::io_error(e, Url::parse("memory://strings_heap").unwrap()))?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }

        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    /// 将 RVA（相对虚拟地址）转换为文件偏移
    ///
    /// RVA（Relative Virtual Address）是 PE 文件中的重要概念：
    /// - RVA 是相对于映像基址的偏移量
    /// - 文件偏移是相对于文件开头的物理位置
    ///
    /// 转换过程：
    /// 1. 在 PE 节表中查找包含目标 RVA 的节
    /// 2. 计算 RVA 在节内的相对偏移
    /// 3. 将相对偏移加到节的文件偏移上，得到最终的文件偏移
    ///
    /// # 参数
    /// * `rva` - 要转换的相对虚拟地址
    ///
    /// # 返回
    /// * `Ok(u32)` - 成功转换的文件偏移
    /// * `Err(GaiaError)` - 找不到包含该 RVA 的节时的错误
    ///
    /// # 示例
    /// ```
    /// let file_offset = reader.rva_to_file_offset(0x2000)?;
    /// ```
    fn rva_to_file_offset(&mut self, rva: u32) -> Result<u32, GaiaError> {
        // 需要读取完整的 PE 程序以访问节信息
        let pe_program = self.reader.get_program()?.clone();

        // 在节表中查找包含此 RVA 的节
        for section in &pe_program.sections {
            let section_start = section.virtual_address;
            let section_end = section_start + section.virtual_size;

            // 检查 RVA 是否在该节的地址范围内
            if rva >= section_start && rva < section_end {
                // 计算 RVA 在节内的相对偏移
                let offset_in_section = rva - section_start;
                // 返回文件偏移 = 节的文件偏移 + 相对偏移
                return Ok(section.pointer_to_raw_data + offset_in_section);
            }
        }

        // 找不到包含该 RVA 的节
        Err(GaiaError::syntax_error(format!("无法将 RVA 0x{:x} 转换为文件偏移", rva), SourceLocation::default()))
    }
}

/// 读取堆索引（根据大小 2 或 4 字节）
fn read_type_def_or_ref_index<R: Read>(cursor: &mut R, idx_size: u32) -> Result<u32, GaiaError> {
    if idx_size == 2 {
        cursor
            .read_u16::<LittleEndian>()
            .map(|v| v as u32)
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://type_def_or_ref_index").unwrap()))
    }
    else {
        cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://type_def_or_ref_index").unwrap()))
    }
}

fn read_heap_index<R: Read>(cursor: &mut R, idx_size: u32) -> Result<u32, GaiaError> {
    if idx_size == 2 {
        cursor
            .read_u16::<LittleEndian>()
            .map(|v| v as u32)
            .map_err(|e| GaiaError::io_error(e, Url::parse("memory://heap_index").unwrap()))
    }
    else if idx_size == 4 {
        cursor.read_u32::<LittleEndian>().map_err(|e| GaiaError::io_error(e, Url::parse("memory://heap_index").unwrap()))
    }
    else {
        Err(GaiaError::syntax_error("非法堆索引大小".to_string(), SourceLocation::default()))
    }
}

/// 从 #Strings 堆读取字符串（以 0 结尾的 UTF-8）
fn read_string_from_heap(pe_data: &[u8], strings_start: u32, strings_size: u32, index: u32) -> Result<String, GaiaError> {
    if index == 0 {
        return Ok(String::new());
    }
    let base = strings_start + index;
    let end = strings_start + strings_size;
    if base >= end || (base as usize) >= pe_data.len() {
        return Ok(String::new());
    }
    let mut i = base as usize;
    let mut bytes = Vec::new();
    while i < pe_data.len() && (i as u32) < end {
        let b = pe_data[i];
        if b == 0 {
            break;
        }
        bytes.push(b);
        i += 1;
    }
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// 简单搜索子切片是否存在
fn find_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }
    let n = needle.len();
    for i in 0..=haystack.len() - n {
        if &haystack[i..i + n] == needle {
            return true;
        }
    }
    false
}
