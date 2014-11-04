pub use self::compilation_target::{AbiCompatible, Architecture, CompilationTarget, ApiCompatible};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
pub use url::Url;
mod compilation_target;
