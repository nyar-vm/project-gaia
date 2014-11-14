use crate::program::{
    ClrAccessFlags, ClrHeader, ClrMethod, ClrProgram, ClrType, ClrTypeReference, ClrVersion, DotNetAssemblyInfo,
    MetadataHeader, StreamHeader,
};
use byteorder::{LittleEndian, ReadBytesExt};
use gaia_types::{GaiaDiagnostics, GaiaError, SourceLocation};
use pe_assembler::viewer::PeView;
use std::{
    fs,
    io::{Cursor, Read, Seek, SeekFrom},
};
use url::Url;

/// .NET PE 文件惰性读取器
///
/// 该类负责读取和解析 .NET 程序集文件，提供以下功能：
/// - 检查文件是否为有效的 .NET 程序集
/// - 解析 CLR 头和元数据
/// - 提取程序集的基本信息
/// - 验证程序集的完整性
/// - 支持惰性读取和完整解析两种模式
#[derive(Debug)]
pub struct DotNetReader {
    /// 整个 PE 文件的原始字节数据
    pe_data: Vec<u8>,
    /// PE 文件的结构化视图，提供对 PE 各部分的访问
    pe_view: PeView,
    /// 解析后的 CLR 头信息
    clr_header: Option<ClrHeader>,
    /// 解析后的元数据头信息
    metadata_header: Option<MetadataHeader>,
    /// 元数据流头信息列表
    stream_headers: Vec<StreamHeader>,
    /// 提取的程序集基本信息
    assembly_info: Option<DotNetAssemblyInfo>,
    /// 完整解析的 CLR 程序（惰性加载）
    clr_program: Option<ClrProgram>,
}

impl DotNetReader {
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
    pub fn read_from_file(file_path: &str) -> Result<Self, GaiaError> {
        // 将整个 PE 文件读入内存
        let pe_data =
            fs::read(file_path).map_err(|e| GaiaError::io_error(e, Url::parse(&format!("file://{}", file_path)).unwrap()))?;

        // 创建 PE 视图以访问 PE 结构
        let pe_view = PeView::view_bytes(&pe_data)?;

        // 创建读取器实例
        let mut reader = DotNetReader {
            pe_data,
            pe_view,
            clr_header: None,
            metadata_header: None,
            stream_headers: Vec::new(),
            assembly_info: None,
            clr_program: None,
        };

        // 执行解析工作流程
        reader.parse_clr_header()?;
        reader.parse_metadata()?;
        reader.extract_assembly_info()?;

        Ok(reader)
    }

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
    pub fn is_dotnet_assembly(file_path: &str) -> Result<bool, GaiaError> {
        // 读取 PE 文件
        let pe_data =
            fs::read(file_path).map_err(|e| GaiaError::io_error(e, Url::parse(&format!("file://{}", file_path)).unwrap()))?;

        // 创建 PE 视图
        let pe_view = PeView::view_bytes(&pe_data)?;

        // 需要读取完整的 PE 程序以访问数据目录
        let pe_program = pe_view.to_program()?;

        // 检查 CLR 数据目录是否存在（索引 14 是 CLR 运行时头）
        // .NET 程序集必须包含此数据目录
        if let Some(clr_dir) = pe_program.header.optional_header.data_directories.get(14) {
            Ok(clr_dir.virtual_address != 0 && clr_dir.size != 0)
        }
        else {
            Ok(false)
        }
    }

    /// 惰性读取程序集基本信息
    ///
    /// 仅读取程序集的基本标识信息，不解析完整的类型系统。
    /// 适用于快速获取程序集名称、版本等信息的场景。
    ///
    /// # 返回
    /// * `Ok(DotNetAssemblyInfo)` - 程序集基本信息
    /// * `Err(GaiaError)` - 读取过程中的错误
    pub fn get_assembly_info(&self) -> Result<DotNetAssemblyInfo, GaiaError> {
        if let Some(ref info) = self.assembly_info {
            Ok(info.clone())
        }
        else {
            Err(GaiaError::syntax_error("程序集信息未解析".to_string(), SourceLocation::default()))
        }
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
    pub fn validate_assembly(&self) -> Result<Vec<String>, GaiaError> {
        let mut warnings = Vec::new();

        // 验证 CLR 头 - 必需的核心头信息
        if self.clr_header.is_none() {
            warnings.push("缺少 CLR 头".to_string());
        }

        // 验证元数据头 - 描述类型系统的元数据
        if self.metadata_header.is_none() {
            warnings.push("缺少元数据头".to_string());
        }

        // 验证流头 - 包含实际的元数据流
        if self.stream_headers.is_empty() {
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
    pub fn get_assembly_summary(&self) -> String {
        if let Some(ref info) = self.assembly_info {
            format!(
                "程序集: {}\n版本: {}\n文化: {}\n公钥标记: {}\n运行时版本: {}",
                info.name,
                info.version,
                info.culture.as_deref().unwrap_or("neutral"),
                info.public_key_token.as_deref().unwrap_or("null"),
                info.runtime_version.as_deref().unwrap_or("unknown")
            )
        }
        else {
            "无法获取程序集信息".to_string()
        }
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
            self.stream_headers = self.read_stream_headers(metadata_offset)?;
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
        // TODO: 实现从元数据中提取程序集信息的逻辑
        // 这需要解析 #Strings 流来获取程序集名称
        // 解析 Assembly 表来获取版本信息
        // 解析 Culture 和 PublicKey 信息

        // 临时实现，后续需要完善
        self.assembly_info = Some(DotNetAssemblyInfo {
            name: "Unknown".to_string(),
            version: "0.0.0.0".to_string(),
            culture: None,
            public_key_token: None,
            runtime_version: Some("v4.0.30319".to_string()),
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
    fn parse_full_program(&self) -> Result<ClrProgram, GaiaError> {
        // 创建基本的 CLR 程序结构
        let mut program = ClrProgram::new("UnknownAssembly".to_string());

        // 设置版本信息
        program.version = ClrVersion { major: 1, minor: 0, build: 0, revision: 0 };

        // 设置访问标志
        program.access_flags =
            ClrAccessFlags { is_public: true, is_private: false, is_security_transparent: false, is_retargetable: false };

        // TODO: 实现完整的元数据表解析
        // 这需要解析以下元数据表：
        // - Assembly 表：程序集信息
        // - AssemblyRef 表：外部程序集引用
        // - Module 表：模块信息
        // - TypeDef 表：类型定义
        // - MethodDef 表：方法定义
        // - FieldDef 表：字段定义
        // - MemberRef 表：成员引用
        // - TypeRef 表：类型引用
        // - Param 表：参数信息
        // - Property 表：属性信息
        // - Event 表：事件信息
        // - CustomAttribute 表：自定义属性

        // 临时添加一个示例类型
        let mut example_type = ClrType::new("ExampleClass".to_string(), Some("ExampleNamespace".to_string()));

        // 添加示例方法
        let void_type = ClrTypeReference {
            name: "Void".to_string(),
            namespace: Some("System".to_string()),
            assembly: Some("mscorlib".to_string()),
            is_value_type: true,
            is_reference_type: false,
            generic_parameters: Vec::new(),
        };

        let example_method = ClrMethod::new("ExampleMethod".to_string(), void_type);
        example_type.add_method(example_method);

        program.add_type(example_type);

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
    fn find_and_read_clr_header(&self) -> Result<Option<ClrHeader>, GaiaError> {
        // 获取 PE 程序以访问数据目录
        let pe_program = self.pe_view.to_program()?;

        // 检查 CLR 数据目录是否存在（索引 14 是 CLR 运行时头）
        if let Some(clr_dir) = pe_program.header.optional_header.data_directories.get(14) {
            if clr_dir.virtual_address == 0 || clr_dir.size == 0 {
                return Ok(None);
            }

            // 将 RVA 转换为文件偏移
            let file_offset = self.rva_to_file_offset(clr_dir.virtual_address)?;

            // 读取 CLR 头
            let mut cursor = Cursor::new(&self.pe_data);
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
    fn read_metadata_header(&self, offset: u32) -> Result<MetadataHeader, GaiaError> {
        let mut cursor = Cursor::new(&self.pe_data);
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
    fn read_stream_headers(&self, metadata_offset: u32) -> Result<Vec<StreamHeader>, GaiaError> {
        let mut stream_headers = Vec::new();

        if let Some(ref metadata_header) = self.metadata_header {
            let mut cursor = Cursor::new(&self.pe_data);
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
                let current_pos = cursor.position();
                let aligned_pos = (current_pos + 3) & !3;
                cursor
                    .seek(SeekFrom::Start(aligned_pos))
                    .map_err(|e| GaiaError::io_error(e, Url::parse("memory://stream_headers").unwrap()))?;

                stream_headers.push(StreamHeader { offset, size, name });
            }
        }

        Ok(stream_headers)
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
    fn rva_to_file_offset(&self, rva: u32) -> Result<u32, GaiaError> {
        // 需要读取完整的 PE 程序来访问节信息
        let pe_program = self.pe_view.to_program()?;

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

/// 从 .NET 程序集文件读取并解析为 CLR 程序
///
/// 这是一个便利函数，用于一次性读取和解析 .NET 程序集文件。
///
/// # 参数
/// * `file_path` - .NET 程序集文件路径
///
/// # 返回
/// * `Ok(ClrProgram)` - 成功解析的 CLR 程序
/// * `Err(GaiaError)` - 读取或解析过程中的错误
pub fn read_dotnet_assembly(file_path: &str) -> GaiaDiagnostics<ClrProgram> {
    match DotNetReader::read_from_file(file_path) {
        Ok(mut reader) => match reader.to_clr_program() {
            Ok(program) => GaiaDiagnostics::success(program),
            Err(error) => GaiaDiagnostics::failure(error),
        },
        Err(error) => GaiaDiagnostics::failure(error),
    }
}
