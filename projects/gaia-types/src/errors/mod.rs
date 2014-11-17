#![doc = include_str!("readme.md")]

pub use self::diagnostics::GaiaDiagnostics;
use crate::{
    helpers::{Architecture, CompilationTarget},
    SourceLocation,
};
use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    panic::Location,
};
use tracing::Level;
use url::Url;

mod convert;
mod diagnostics;
mod display;

/// 本crate的结果类型，使用GaiaError作为错误类型
///
/// 这个类型别名简化了错误处理，所有可能返回错误的函数都应该使用这个类型
pub type Result<T> = std::result::Result<T, GaiaError>;

/// Gaia错误类型，包装了具体的错误种类[GaiaErrorKind]
///
/// 使用Box来减少枚举的大小，提高性能

pub struct GaiaError {
    level: Level,
    /// 具体的错误种类，使用Box包装以减少内存占用
    ///
    /// 这个字段包含了实际的错误信息，通过Box指针间接存储，
    /// 这样可以避免在栈上分配较大的枚举值，提高性能
    kind: Box<GaiaErrorKind>,
}

/// Gaia 错误种类枚举，定义了所有可能的错误类型
#[derive(Debug)]
pub enum GaiaErrorKind {
    /// 无效指令错误，当解析到未知或不支持的指令时使用
    InvalidInstruction {
        /// 无效的指令字符串
        instruction: String,
        /// 指令所属的架构
        architecture: Architecture,
    },
    /// 不支持的架构错误，当尝试在不支持的架构上执行操作时使用
    UnsupportedArchitecture {
        /// 不支持的架构
        architecture: Architecture,
    },
    /// 无效范围错误，当实际长度与期望长度不匹配时使用
    ///
    /// 这种错误通常发生在解析二进制数据或验证数据结构时，
    /// 当实际数据长度与期望的长度不符时抛出此错误。
    InvalidRange {
        /// 实际长度
        ///
        /// 表示实际测量或解析得到的数据长度。
        length: usize,
        /// 期望长度
        ///
        /// 表示根据规范或预期应该具有的长度。
        expect: usize,
    },
    /// IO 错误，包含底层的 IO 错误和可选的 URL 信息
    ///
    /// 当文件读写、网络请求等 IO 操作失败时使用
    IoError {
        /// 底层的 IO 错误
        ///
        /// 包含了具体的 IO 错误信息，如文件不存在、权限不足等。
        io_error: std::io::Error,
        /// 与 IO 操作相关的 URL，可选
        ///
        /// 如果 IO 操作与特定文件或网络资源相关，这里存储其 URL。
        /// 可以是文件系统路径或网络地址。
        url: Option<Url>,
    },
    /// 语法错误，包含错误消息和源代码位置信息
    ///
    /// 当解析源代码发现语法问题时使用，提供详细的错误位置信息
    SyntaxError {
        /// 错误消息，描述具体的语法问题
        ///
        /// 包含了人类可读的语法错误描述，如 "缺少分号"、"未闭合的括号" 等。
        message: String,
        /// 错误发生的源代码位置信息
        ///
        /// 包含了错误所在的文件、行号、列号等位置信息，
        /// 帮助开发者快速定位问题。
        location: SourceLocation,
    },
    /// 停止运行
    StageError {
        /// 停止运行的地方
        location: Location<'static>,
    },
    /// 功能未实现错误
    ///
    /// 当调用尚未实现的功能时使用
    NotImplemented {
        /// 未实现功能的描述
        feature: String,
    },
    /// 不支持的功能错误
    ///
    /// 当尝试使用不支持的功能时使用
    UnsupportedFeature {
        /// 不支持的功能描述
        feature: String,
        /// 错误发生的源代码位置信息
        location: SourceLocation,
    },
    /// 自定义错误，包含自定义的错误消息
    ///
    /// 当需要表示特定业务逻辑错误或其他非标准错误时使用
    CustomError {
        /// 自定义错误消息
        message: String,
    },
    /// 适配器错误，当适配器操作失败时使用
    ///
    /// 包含适配器名称和具体的错误信息
    AdapterError {
        /// 适配器名称
        adapter_name: String,
        /// 错误消息
        message: String,
        /// 可选的源错误
        source: Option<Box<GaiaError>>,
    },
    /// 平台不支持错误，当目标平台不支持某个操作时使用
    ///
    /// 包含平台名称和不支持的操作描述
    PlatformUnsupported {
        /// 平台名称
        platform: String,
        /// 不支持的操作描述
        operation: String,
    },
    /// 配置错误，当配置文件解析或验证失败时使用
    ///
    /// 包含配置文件路径和错误信息
    ConfigError {
        /// 配置文件路径
        config_path: Option<String>,
        /// 错误消息
        message: String,
    },
    /// 不支持的编译目标错误
    ///
    /// 当尝试编译到不支持的目标平台时使用
    UnsupportedTarget {
        /// 不支持的编译目标
        target: CompilationTarget,
    },
    /// 编译失败错误
    ///
    /// 当编译过程中发生错误时使用
    CompilationFailed {
        /// 编译目标
        target: CompilationTarget,
        /// 错误消息
        message: String,
    },

    /// 保存错误，当保存文件失败时使用
    ///
    /// 包含保存格式和错误消息
    SaveError {
        /// 保存格式
        format: String,
        /// 错误消息
        message: String,
    },
}

impl GaiaError {
    /// 创建一个语法错误
    ///
    /// 当源代码解析过程中发现语法问题时使用此函数创建错误
    ///
    /// # 参数
    ///
    /// * `message` - 错误消息，描述具体的语法问题
    /// * `location` - 错误发生的源代码位置信息
    ///
    /// # 返回值
    ///
    /// 返回一个包含语法错误信息的GaiaError实例
    ///
    /// # 示例
    ///
    /// ```
    /// use gaia_types::{GaiaError, SourceLocation};
    /// let location = SourceLocation::default();
    /// let error = GaiaError::syntax_error("缺少分号", location);
    /// ```
    pub fn syntax_error(message: impl ToString, location: SourceLocation) -> Self {
        GaiaErrorKind::SyntaxError { message: message.to_string(), location }.into()
    }

    /// 创建一个IO错误
    ///
    /// 当文件读写、网络请求等IO操作失败时使用此函数创建错误
    ///
    /// # 参数
    ///
    /// * `io_error` - 底层的IO错误
    /// * `url` - 与IO操作相关的URL（如文件路径或网络地址）
    ///
    /// # 返回值
    ///
    /// 返回一个包含IO错误信息的GaiaError实例
    ///
    /// # 示例
    ///
    /// ```
    /// use gaia_types::GaiaError;
    /// use url::Url;
    /// let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
    /// let url = Url::from_file_path("/path/to/file")
    ///     .ok()
    ///     .and_then(|x| Some(x))
    ///     .unwrap_or_else(|| Url::parse("file:///path/to/file").unwrap());
    /// let error = GaiaError::io_error(io_err, url);
    /// ```
    pub fn io_error(io_error: std::io::Error, url: Url) -> Self {
        GaiaErrorKind::IoError { io_error, url: Some(url) }.into()
    }

    /// 创建一个无效指令错误
    ///
    /// 当解析到未知或不支持的指令时使用此函数创建错误
    ///
    /// # 参数
    ///
    /// * `instruction` - 无效的指令字符串
    /// * `architecture` - 指令所属的架构
    ///
    /// # 返回值
    ///
    /// 返回一个包含无效指令错误信息的GaiaError实例
    ///
    /// # 示例
    ///
    /// ```
    /// use gaia_types::{helpers::Architecture, GaiaError};
    /// let error = GaiaError::invalid_instruction("未知指令", Architecture::X86);
    /// ```
    pub fn invalid_instruction(instruction: impl ToString, architecture: Architecture) -> Self {
        GaiaErrorKind::InvalidInstruction { instruction: instruction.to_string(), architecture }.into()
    }

    /// 创建一个不支持的架构错误
    ///
    /// 当尝试在不支持的架构上执行操作时使用此函数创建错误
    ///
    /// # 参数
    ///
    /// * `architecture` - 不支持的架构
    ///
    /// # 返回值
    ///
    /// 返回一个包含不支持的架构错误信息的GaiaError实例
    ///
    /// # 示例
    ///
    /// ```
    /// use gaia_types::{helpers::Architecture, GaiaError};
    /// let error = GaiaError::unsupported_architecture(Architecture::ARM32);
    /// ```
    pub fn unsupported_architecture(architecture: Architecture) -> Self {
        GaiaErrorKind::UnsupportedArchitecture { architecture }.into()
    }

    /// 创建一个无效范围错误
    ///
    /// 当实际数据长度与期望长度不匹配时使用此函数创建错误
    ///
    /// # 参数
    ///
    /// * `length` - 实际长度
    /// * `expect` - 期望长度
    ///
    /// # 返回值
    ///
    /// 返回一个包含无效范围错误信息的GaiaError实例
    ///
    /// # 示例
    ///
    /// ```
    /// use gaia_types::GaiaError;
    /// let error = GaiaError::invalid_range(1024, 2048);
    /// ```
    pub fn invalid_range(length: usize, expect: usize) -> Self {
        GaiaErrorKind::InvalidRange { length, expect }.into()
    }

    /// 创建一个无效数据错误。
    ///
    /// # 参数
    ///
    /// * `data` - 无效数据的描述。
    ///
    /// # 返回值
    ///
    /// 返回一个包含无效数据错误信息的GaiaError实例。
    pub fn invalid_data(data: impl ToString) -> Self {
        Self {
            level: Level::ERROR,
            kind: Box::new(GaiaErrorKind::CustomError { message: format!("无效数据: {}", data.to_string()) }),
        }
    }

    /// 创建一个无效魔术头错误。
    ///
    /// # 参数
    ///
    /// * `head` - 实际的魔术头。
    /// * `expect` - 期望的魔术头。
    ///
    /// # 返回值
    ///
    /// 返回一个包含无效魔术头错误信息的GaiaError实例。
    pub fn invalid_magic_head(head: Vec<u8>, expect: Vec<u8>) -> Self {
        Self {
            level: Level::ERROR,
            kind: Box::new(GaiaErrorKind::CustomError {
                message: format!("无效数据头: {:?}, 期望: {:?}", head, expect)
            }),
        }
    }

    /// 返回错误的种类。
    pub fn kind(&self) -> &GaiaErrorKind {
        &self.kind
    }

    /// 返回错误的级别。
    pub fn level(&self) -> &Level {
        &self.level
    }

    /// 创建一个功能未实现错误
    ///
    /// 当调用尚未实现的功能时使用此函数创建错误
    ///
    /// # 参数
    ///
    /// * `feature` - 未实现功能的描述
    ///
    /// # 返回值
    ///
    /// 返回一个包含功能未实现错误信息的GaiaError实例
    ///
    /// # 示例
    ///
    /// ```
    /// use gaia_types::GaiaError;
    /// let error = GaiaError::not_implemented("PE context creation");
    /// ```
    pub fn not_implemented(feature: impl ToString) -> Self {
        GaiaErrorKind::NotImplemented { feature: feature.to_string() }.into()
    }

    /// 创建适配器错误
    ///
    /// # 参数
    /// * `adapter_name` - 适配器名称
    /// * `message` - 错误消息
    /// * `source` - 可选的源错误
    ///
    /// # 示例
    /// ```
    /// use gaia_types::GaiaError;
    /// let error = GaiaError::adapter_error("PeExportAdapter", "导出失败", None);
    /// ```
    pub fn adapter_error(adapter_name: impl ToString, message: impl ToString, source: Option<Box<GaiaError>>) -> Self {
        GaiaErrorKind::AdapterError { adapter_name: adapter_name.to_string(), message: message.to_string(), source }.into()
    }

    /// 创建平台不支持错误
    ///
    /// # 参数
    /// * `platform` - 平台名称
    /// * `operation` - 不支持的操作描述
    ///
    /// # 示例
    /// ```
    /// use gaia_types::GaiaError;
    /// let error = GaiaError::platform_unsupported("WASI", "内联汇编");
    /// ```
    pub fn platform_unsupported(platform: impl ToString, operation: impl ToString) -> Self {
        GaiaErrorKind::PlatformUnsupported { platform: platform.to_string(), operation: operation.to_string() }.into()
    }

    /// 创建配置错误
    ///
    /// # 参数
    /// * `config_path` - 可选的配置文件路径
    /// * `message` - 错误消息
    ///
    /// # 示例
    /// ```
    /// use gaia_types::GaiaError;
    /// let error = GaiaError::config_error(Some("config.toml"), "配置文件格式错误");
    /// ```
    pub fn config_error(config_path: Option<impl ToString>, message: impl ToString) -> Self {
        GaiaErrorKind::ConfigError { config_path: config_path.map(|p| p.to_string()), message: message.to_string() }.into()
    }

    /// 创建不支持的编译目标错误
    ///
    /// # 参数
    /// * `target` - 不支持的编译目标
    ///
    /// # 示例
    /// ```
    /// use gaia_types::{
    ///     helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    ///     GaiaError,
    /// };
    /// let target = CompilationTarget {
    ///     build: Architecture::X86_64,
    ///     host: AbiCompatible::ELF,
    ///     target: ApiCompatible::Gnu,
    /// };
    /// let error = GaiaError::unsupported_target(target);
    /// ```
    pub fn unsupported_target(target: CompilationTarget) -> Self {
        GaiaErrorKind::UnsupportedTarget { target }.into()
    }

    /// 创建编译失败错误
    ///
    /// # 参数
    /// * `target` - 编译目标
    /// * `message` - 错误消息
    ///
    /// # 示例
    /// ```
    /// use gaia_types::{
    ///     helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    ///     GaiaError,
    /// };
    /// let target = CompilationTarget {
    ///     build: Architecture::X86_64,
    ///     host: AbiCompatible::ELF,
    ///     target: ApiCompatible::Gnu,
    /// };
    /// let error = GaiaError::compilation_failed(target, "无法生成字节码");
    /// ```
    pub fn compilation_failed(target: CompilationTarget, message: impl ToString) -> Self {
        GaiaErrorKind::CompilationFailed { target, message: message.to_string() }.into()
    }

    /// 创建一个保存错误。
    ///
    /// # 参数
    ///
    /// * `format` - 保存格式。
    /// * `message` - 错误消息。
    ///
    /// # 返回值
    ///
    /// 返回一个包含保存错误信息的GaiaError实例。
    pub fn save_error(format: impl ToString, message: impl ToString) -> Self {
        GaiaErrorKind::SaveError { format: format.to_string(), message: message.to_string() }.into()
    }
    /// 创建一个不支持的功能错误。
    ///
    /// # 参数
    ///
    /// * `p0` - 不支持的功能描述。
    /// * `p1` - 错误发生的源代码位置信息。
    ///
    /// # 返回值
    ///
    /// 返回一个包含不支持的功能错误信息的GaiaError实例。
    pub fn unsupported_feature(feature: impl ToString, location: SourceLocation) -> Self {
        GaiaErrorKind::UnsupportedFeature { feature: feature.to_string(), location }.into()
    }
    /// 创建一个自定义错误。
    ///
    /// # 参数
    ///
    /// * `message` - 错误消息。
    ///
    /// # 返回值
    ///
    /// 返回一个包含自定义错误信息的GaiaError实例。
    pub fn custom_error(message: String) -> GaiaError {
        GaiaErrorKind::CustomError { message }.into()
    }
}
