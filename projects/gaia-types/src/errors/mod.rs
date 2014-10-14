use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

mod convert;
mod display;

/// The result type of this crate.
pub type Result<T> = std::result::Result<T, GaiaError>;

/// A boxed error kind, wrapping an [GaiaErrorKind].
#[derive(Clone)]
pub struct GaiaError {
    kind: Box<GaiaErrorKind>,
}

/// The kind of [GaiaError].
#[derive(Debug, Copy, Clone)]
pub enum GaiaErrorKind {
    /// An unknown error.
    SyntaxError,
}

