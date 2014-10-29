/// Import 适配器模块
///
/// 负责从各个平台的指令格式导入到 Gaia 统一指令格式
/// 使用对象传递而非字符串拼接
use crate::instruction::*;
use gaia_types::*;

pub mod il_import;
pub mod jvm_import;
pub mod pe_import;
pub mod wasi_import;

pub use il_import::*;
pub use jvm_import::*;
pub use pe_import::*;
pub use wasi_import::*;

/// Import 适配器 trait
///
/// 定义了从平台特定格式导入到 Gaia 格式的接口
pub trait ImportAdapter<T> {
    /// 从平台特定的指令转换为 Gaia 指令
    fn import_instruction(&self, platform_instruction: &T) -> Result<GaiaInstruction>;

    /// 从平台特定的程序转换为 Gaia 程序
    fn import_program(&self, platform_program: &[T]) -> Result<GaiaProgram>;

    /// 获取适配器名称
    fn adapter_name(&self) -> &'static str;
}

/// Import 适配器管理器
#[derive(Debug)]
pub struct ImportAdapterManager {
    pe_adapter: PeImportAdapter,
    il_adapter: IlImportAdapter,
    jvm_adapter: JvmImportAdapter,
    wasi_adapter: WasiImportAdapter,
}

impl ImportAdapterManager {
    /// 创建新的 Import 适配器管理器
    pub fn new() -> Self {
        Self {
            pe_adapter: PeImportAdapter::new(),
            il_adapter: IlImportAdapter::new(),
            jvm_adapter: JvmImportAdapter::new(),
            wasi_adapter: WasiImportAdapter::new(),
        }
    }

    /// 获取 PE Import 适配器
    pub fn pe(&self) -> &PeImportAdapter {
        &self.pe_adapter
    }

    /// 获取 IL Import 适配器
    pub fn il(&self) -> &IlImportAdapter {
        &self.il_adapter
    }

    /// 获取 JVM Import 适配器
    pub fn jvm(&self) -> &JvmImportAdapter {
        &self.jvm_adapter
    }

    /// 获取 WASI Import 适配器
    pub fn wasi(&self) -> &WasiImportAdapter {
        &self.wasi_adapter
    }
}

impl Default for ImportAdapterManager {
    fn default() -> Self {
        Self::new()
    }
}
