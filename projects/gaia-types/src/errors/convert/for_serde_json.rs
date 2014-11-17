use crate::GaiaError;

impl From<serde_json::Error> for GaiaError {
    fn from(error: serde_json::Error) -> Self {
        GaiaError::invalid_data(error)
    }
}
