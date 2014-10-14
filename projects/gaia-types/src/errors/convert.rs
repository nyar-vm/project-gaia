use super::*;

impl From<GaiaErrorKind> for GaiaError {
    fn from(value: GaiaErrorKind) -> Self {
        Self {
            kind: Box::new(value),
        }
    }
}