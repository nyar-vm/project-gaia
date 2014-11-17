/// 统一的适配器接口定义, 替代 import 和 export adapter
pub mod adapters;
pub mod assembler;
pub mod backends;
/// 配置管理模块
pub mod config;
/// Gaia Universal Assembler
///
/// 通用汇编器，支持多平台指令集转换
/// 使用对象传递而非字符串拼接，复用现有项目的类型定义
pub mod instruction;
