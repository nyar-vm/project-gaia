use crate::{reader::Token, GaiaError};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct TokenStream<'input, T> {
    pub raw: &'input str,
    pub tokens: Cursor<Vec<T>>,
}

impl<'input, T> TokenStream<'input, T> {
    pub fn new(raw: &'input str, tokens: Vec<T>) -> Self {
        Self { raw, tokens: Cursor::new(tokens) }
    }
    pub fn current(&self) -> Result<Token<T>, GaiaError> {
        todo!()
    }

    pub fn get_text(&self) -> Result<&'input str, GaiaError> {
        let token = self.current()?;
        match self.raw.get(token.get_range()) {
            Some(s) => Ok(s),
            None => Err(GaiaError::invalid_range(self.raw.len(), token.position.offset + token.position.length)),
        }
    }
}
