use crate::GaiaError;
use std::{
    fmt::Debug,
    ops::{ControlFlow, FromResidual, Try},
    panic::Location,
};
use tracing::Level;

#[derive(Debug)]
pub struct GaiaDiagnostics<T> {
    /// 包含实际结果的结果类型，可能是成功值或 Fatal 错误值
    pub result: Result<T, GaiaError>,
    /// 包含诊断信息的错误向量
    ///
    /// 当操作失败时，可能会包含多个诊断错误，每个错误都包含了具体的错误信息和位置
    pub diagnostics: Vec<GaiaError>,
}

impl<T> GaiaDiagnostics<T> {
    pub fn success(value: T) -> Self {
        Self { result: Ok(value), diagnostics: Vec::new() }
    }

    pub fn failure(fatal: GaiaError) -> Self {
        Self { result: Err(fatal), diagnostics: Vec::new() }
    }

    pub fn add_warning(&mut self, warnings: impl Into<GaiaError>) {
        let mut error = warnings.into();
        error.level = Level::WARN;
        self.diagnostics.push(error);
    }

    pub fn add_tracing(&mut self, tracing: impl Into<GaiaError>) {
        let mut error = tracing.into();
        error.level = Level::TRACE;
        self.diagnostics.push(error);
    }

    /// 检查是否应该中断执行
    ///
    /// 规则：
    /// - Fatal 错误（result 是 Err）-> 中断
    /// - 诊断中有 Error 级别的错误 -> 中断  
    /// - 其他情况 -> 可以继续
    pub fn should_halt(&self) -> bool {
        if self.result.is_err() {
            return true;
        }

        self.diagnostics.iter().any(|diag| diag.level == Level::ERROR)
    }

    /// 获取结果值，如果应该中断则返回 ControlFlow::Break
    pub fn try_value(self) -> ControlFlow<Self, T> {
        if self.should_halt() {
            ControlFlow::Break(self)
        }
        else {
            match self.result {
                Ok(value) => ControlFlow::Continue(value),
                Err(_) => ControlFlow::Break(self), // 这行理论上不会执行，因为 should_halt 已经检查了
            }
        }
    }
}

impl<T, U> FromResidual<GaiaDiagnostics<T>> for GaiaDiagnostics<U> {
    #[track_caller]
    fn from_residual(residual: GaiaDiagnostics<T>) -> Self {
        match residual.result {
            // 没有致命错误， 检查诊断中是否有错误级别的问题
            Ok(_) => {
                let has_error = residual.diagnostics.iter().any(|diag| diag.level == Level::ERROR);
                // 有错误，创建 stage error
                if has_error {
                    let location = Location::caller();
                    Self { result: Err(GaiaError::stage_error(*location)), diagnostics: residual.diagnostics }
                }
                // 这应该是一个逻辑错误，因为成功的情况不应该进入 from_residual
                else {
                    unreachable!()
                }
            }
            // 已经有致命错误，直接传递错误和诊断信息
            Err(e) => Self { result: Err(e), diagnostics: residual.diagnostics },
        }
    }
}

impl<T> Try for GaiaDiagnostics<T> {
    type Output = T;
    type Residual = GaiaDiagnostics<T>;

    fn from_output(output: Self::Output) -> Self {
        Self::success(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        self.try_value()
    }
}
