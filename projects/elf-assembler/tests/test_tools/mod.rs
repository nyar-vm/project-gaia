//! Easy test module for PE file analysis
//!
//! This module provides utilities for automated testing of PE file analysis,
//! including expectation generation, validation, and test organization.

use gaia_types::GaiaError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

/// PE 文件期望结构体 - 用于定义测试期望
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElfExpected {
    /// 期望的文件名
    pub file_name: String,
    /// 期望的架构
    pub target_arch: String,
    /// 期望的子系统类型
    pub subsystem: String,
    /// 期望的导出函数数量
    pub export_count: usize,
    /// 期望的导入函数数量
    pub import_count: usize,
    /// 期望的节数量
    pub section_count: usize,
    /// 期望包含的特定导出函数
    pub expected_exports: Vec<String>,
    /// 期望包含的特定导入函数
    pub expected_imports: Vec<String>,
    /// 期望的节名称
    pub expected_sections: Vec<String>,
    /// 期望的入口点（可选）
    pub entry_point: Option<u32>,
    /// 期望的镜像基址（可选）
    pub image_base: Option<u64>,
    /// 文件路径（用于调试）
    #[serde(skip)]
    pub file_path: PathBuf,
}
