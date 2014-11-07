mod analysis_dll;
mod analysis_exe;

// 重新导出 easy_test 模块中的类型和函数，以保持向后兼容性
#[cfg(feature = "easy-test")]
pub use pe_assembler::easy_test::WindowsDllAnalysis;
