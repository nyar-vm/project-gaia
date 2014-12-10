#![doc = include_str!("readme.md")]

pub use self::diagnostics::GaiaDiagnostics;
use crate::{
    helpers::{Architecture, CompilationTarget},
    reader::SourceLocation,
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

/// Result type for this crate, using GaiaError as the error type
///
/// This type alias simplifies error handling; all functions that might return an error should use this type.
pub type Result<T> = std::result::Result<T, GaiaError>;

/// Gaia error type, wrapping a specific error kind [GaiaErrorKind]
///
/// [Box] is used here to reduce the size of the enum and improve performance.
pub struct GaiaError {
    level: Level,
    /// The specific error kind, wrapped in a Box to reduce memory footprint
    ///
    /// This field contains the actual error information, stored indirectly via a Box pointer.
    /// This avoids allocating large enum values on the stack, thereby improving performance.
    kind: Box<GaiaErrorKind>,
}

/// Gaia error kind enum, defining all possible error types
#[derive(Debug)]
pub enum GaiaErrorKind {
    /// Invalid instruction error, used when an unknown or unsupported instruction is encountered.
    InvalidInstruction {
        /// The invalid instruction string
        instruction: String,
        /// The architecture to which the instruction belongs
        architecture: Architecture,
    },
    /// Unsupported architecture error, used when attempting an operation on an unsupported architecture.
    UnsupportedArchitecture {
        /// The unsupported architecture
        architecture: Architecture,
    },
    /// Invalid range error, used when the actual length does not match the expected length.
    ///
    /// This error typically occurs when parsing binary data or validating data structures,
    /// where the actual data length does not match the expected length.
    InvalidRange {
        /// Actual length
        ///
        /// Represents the actual measured or parsed data length.
        length: usize,
        /// Expected length
        ///
        /// Represents the length expected according to specifications or expectations.
        expect: usize,
    },
    /// IO error, containing the underlying IO error and optional URL information.
    ///
    /// Used when file read/write, network request, or other IO operations fail.
    IoError {
        /// The underlying IO error
        ///
        /// Contains specific IO error information, such as file not found, insufficient permissions, etc.
        io_error: std::io::Error,
        /// Optional URL associated with the IO operation
        ///
        /// If the IO operation is associated with a specific file or network resource, its URL is stored here.
        /// It can be a filesystem path or a network address.
        url: Option<Url>,
    },
    /// Syntax error, containing an error message and source code location information.
    ///
    /// Used when source code parsing encounters syntax issues, providing detailed error location information.
    SyntaxError {
        /// Error message describing the specific syntax issue
        ///
        /// Contains a human-readable description of the syntax error, such as "missing semicolon," "unclosed parenthesis," etc.
        message: String,
        /// Source code location information where the error occurred
        ///
        /// Contains location information such as file, line number, and column number where the error occurred,
        /// helping developers quickly locate the problem.
        location: SourceLocation,
    },
    /// Termination of execution
    StageError {
        /// Location where the execution stopped
        location: Location<'static>,
    },
    /// Feature not implemented error
    ///
    /// Used when a function that is not yet implemented is called.
    NotImplemented {
        /// Description of the unimplemented feature
        feature: String,
    },
    /// Unsupported feature error
    ///
    /// Used when attempting to use an unsupported feature.
    UnsupportedFeature {
        /// Description of the unsupported feature
        feature: String,
        /// Source code location information where the error occurred
        location: SourceLocation,
    },
    /// Custom error, containing a custom error message.
    ///
    /// Used when a specific business logic error or other non-standard error needs to be represented.
    CustomError {
        /// Custom error message
        message: String,
    },
    /// Adapter error, used when an adapter operation fails.
    ///
    /// Contains the adapter name and specific error message.
    AdapterError {
        /// Adapter name
        adapter_name: String,
        /// Error message
        message: String,
        /// Optional source error
        source: Option<Box<GaiaError>>,
    },
    /// Platform unsupported error, used when the target platform does not support an operation.
    ///
    /// Contains the platform name and a description of the unsupported operation.
    PlatformUnsupported {
        /// Platform name
        platform: String,
        /// Description of the unsupported operation
        operation: String,
    },
    /// Configuration error, used when configuration file parsing or validation fails.
    ///
    /// Contains the configuration file path and error message.
    ConfigError {
        /// Configuration file path
        config_path: Option<String>,
        /// Error message
        message: String,
    },
    /// Unsupported compilation target error
    ///
    /// Used when attempting to compile for an unsupported target platform.
    UnsupportedTarget {
        /// The unsupported compilation target
        target: CompilationTarget,
    },
    /// Compilation failed error
    ///
    /// Used when an error occurs during the compilation process.
    CompilationFailed {
        /// Compilation target
        target: CompilationTarget,
        /// Error message
        message: String,
    },
    /// Unreachable error, used when program execution reaches an unreachable location.
    UnreachableError {
        /// Source code location information of the unreachable location
        location: Location<'static>,
    },
    /// Save error, used when saving a file fails.
    ///
    /// Contains the save format and error message.
    SaveError {
        /// Save format
        format: String,
        /// Error message
        message: String,
    },
}

impl GaiaError {
    /// Creates a syntax error
    ///
    /// Use this function to create an error when source code parsing encounters a syntax issue.
    ///
    /// # Parameters
    ///
    /// * `message` - Error message describing the specific syntax issue
    /// * `location` - Source code location information where the error occurred
    ///
    /// # Return Value
    ///
    /// Returns a GaiaError instance containing the syntax error information.
    ///
    /// # Example
    ///
    /// ```
    /// # use gaia_types::GaiaError;
    /// # use gaia_types::reader::SourceLocation;
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
    /// # use gaia_types::GaiaError;
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
    /// # use gaia_types::GaiaError;
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
    /// # use gaia_types::GaiaError;
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
    /// # use gaia_types::GaiaError;
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
    /// # use gaia_types::{
    /// #     helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    /// #     GaiaError,
    /// # };
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
    /// # use gaia_types::{
    /// #     helpers::{AbiCompatible, ApiCompatible, Architecture, CompilationTarget},
    /// #     GaiaError,
    /// # };
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
    pub fn custom_error(message: impl ToString) -> GaiaError {
        GaiaErrorKind::CustomError { message: message.to_string() }.into()
    }
    /// 创建一个不可达错误。
    ///
    /// 当程序执行到理论上不可达的代码路径时调用此函数。它会自动捕获调用位置。
    ///
    /// # 返回值
    ///
    /// 返回一个包含不可达错误信息的 `GaiaError` 实例。
    ///
    /// # 示例
    ///
    /// ```
    /// # use gaia_types::GaiaError;
    /// fn example_unreachable() -> Result<(), GaiaError> {
    ///     let value = 1;
    ///     match value {
    ///         1 => Ok(()),
    ///         _ => Err(GaiaError::unreachable()), // 此处理论上不可达
    ///     }
    /// }
    /// let _ = example_unreachable();
    /// ```
    #[track_caller]
    pub fn unreachable() -> Self {
        let location = Location::caller();
        GaiaErrorKind::UnreachableError { location: *location }.into()
    }
}
