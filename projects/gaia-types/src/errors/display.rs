use super::*;

impl Error for GaiaError {}

impl Debug for GaiaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for GaiaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

impl Display for GaiaErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GaiaErrorKind::SyntaxError { message, location } => {
                write!(f, "SyntaxError at {:?}: {}", location.url, message)?;
            }
        }
        Ok(())
    }
}
