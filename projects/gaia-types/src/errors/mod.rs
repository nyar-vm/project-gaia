pub use self::diagnostics::GaiaDiagnostics;
use crate::{helpers::Architecture, SourceLocation};
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
    InvalidInstruction {
        instruction: String,
        architecture: Architecture,
    },
    UnsupportedArchitecture {
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
    StageError {
        location: Location<'static>,
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
    /// use gaia_types::{GaiaError, helpers::Architecture};
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
    /// use gaia_types::{GaiaError, helpers::Architecture};
    /// let error = GaiaError::unsupported_architecture(Architecture::ARM);
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

    pub fn kind(&self) -> &GaiaErrorKind {
        &self.kind
    }

    pub fn level(&self) -> &Level {
        &self.level
    }

    pub fn stage_error(location: Location<'static>) -> Self {
        GaiaErrorKind::StageError { location: location.clone() }.into()
    }
}
