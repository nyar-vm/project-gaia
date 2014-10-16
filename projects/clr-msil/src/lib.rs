#![feature(try_blocks)]
#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("../readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]

use url::Url;

pub mod ast;
#[cfg(feature = "easy-test")]
pub mod easy_test;
pub mod lexer;
pub mod parser;
pub mod writer;

#[derive(Clone, Debug, Default)]
pub struct ReadConfig {
    pub url: Option<Url>,
}
