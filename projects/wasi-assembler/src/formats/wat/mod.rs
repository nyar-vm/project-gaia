#![doc = include_str!("readme.md")]

/// AST (Abstract Syntax Tree) definitions for WAT
pub mod ast;
pub mod compiler;
/// Lexical analyzer for WAT source code
pub mod lexer;
/// Parser for converting tokens to AST
pub mod parser;
/// Writer for converting AST back to WAT text
pub mod writer;

pub use ast::*;
pub use compiler::*;
pub use lexer::*;
pub use parser::*;
pub use writer::*;
