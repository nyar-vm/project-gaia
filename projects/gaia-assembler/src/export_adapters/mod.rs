/// Export 适配器模块
///
/// 负责从 Gaia 统一指令格式导出到各个平台的指令格式
/// 使用对象传递而非字符串拼接
use crate::instruction::*;
use gaia_types::*;

pub mod il_export;
pub mod jvm_export;
pub mod pe_export;
pub mod wasi_export;

pub use il_export::*;
pub use jvm_export::*;
pub use pe_export::*;
pub use wasi_export::*;

/// Export 适配器 trait
///
/// 定义了从 Gaia 格式导出到平台特定格式的接口
pub trait ExportAdapter<T> {
    /// 从 Gaia 指令转换为平台特定的指令
    fn export_instruction(&self, gaia_instruction: &GaiaInstruction) -> Result<T>;

    /// 从 Gaia 程序转换为平台特定的程序
    fn export_program(&self, gaia_program: &GaiaProgram) -> Result<Vec<T>>;

    /// 获取适配器名称
    fn adapter_name(&self) -> &'static str;

    /// 生成最终的二进制输出
    fn generate_binary(&self, platform_program: &[T]) -> Result<Vec<u8>>;
}

/// Export 适配器管理器
#[derive(Debug)]
pub struct ExportAdapterManager {
    pe_adapter: PeExportAdapter,
    il_adapter: IlExportAdapter,
    jvm_adapter: JvmExportAdapter,
    wasi_adapter: WasiExportAdapter,
}

impl ExportAdapterManager {
    /// 创建新的 Export 适配器管理器
    pub fn new() -> Self {
        Self {
            pe_adapter: PeExportAdapter::new(),
            il_adapter: IlExportAdapter::new(),
            jvm_adapter: JvmExportAdapter::new(),
            wasi_adapter: WasiExportAdapter::new(),
        }
    }

    /// 获取 PE Export 适配器
    pub fn pe(&self) -> &PeExportAdapter {
        &self.pe_adapter
    }

    /// 获取 IL Export 适配器
    pub fn il(&self) -> &IlExportAdapter {
        &self.il_adapter
    }

    /// 获取 JVM Export 适配器
    pub fn jvm(&self) -> &JvmExportAdapter {
        &self.jvm_adapter
    }

    /// 获取 WASI Export 适配器
    pub fn wasi(&self) -> &WasiExportAdapter {
        &self.wasi_adapter
    }
}

impl Default for ExportAdapterManager {
    fn default() -> Self {
        Self::new()
    }
}
