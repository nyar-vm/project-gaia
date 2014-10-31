//! Mini Rust 语言实现

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;

#[cfg(test)]
pub mod test_core;

pub use codegen::MiniRustParser;
