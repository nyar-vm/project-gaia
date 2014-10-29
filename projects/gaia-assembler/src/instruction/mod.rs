//! Gaia 指令模块
//!
//! 这个模块重新导出 gaia-types 中定义的核心类型，
//! 为 gaia-assembler 提供统一的指令和类型接口。

// 重新导出 gaia-types 中的类型
pub use gaia_types::{GaiaConstant, GaiaFunction, GaiaInstruction, GaiaProgram, GaiaType};
