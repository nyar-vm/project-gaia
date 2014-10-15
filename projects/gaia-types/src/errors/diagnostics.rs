use crate::GaiaError;

pub struct GaiaDiagnostics<T> {
    /// 包含实际结果的结果类型，可能是成功值或 Fatal 错误值
    pub result: std::result::Result<T, GaiaError>,
    /// 包含诊断信息的错误向量
    ///
    /// 当操作失败时，可能会包含多个诊断错误，每个错误都包含了具体的错误信息和位置
    pub diagnostics: Vec<GaiaError>,
}
