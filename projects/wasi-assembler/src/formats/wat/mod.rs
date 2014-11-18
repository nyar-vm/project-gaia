#![doc = include_str!("readme.md")]

/// AST (Abstract Syntax Tree) definitions for WAT
pub mod ast;
pub mod lexer;
/// Parser for converting tokens to AST
pub mod parser;
/// Writer for converting AST back to WAT text
pub mod writer;
