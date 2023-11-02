use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ErrorIdentificator {
    pub fn_call: &'static str,
    pub arg: &'static str,
}

#[derive(Debug)]
pub enum GeneralError {
    NotFound(ErrorIdentificator),
}

impl Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneralError::NotFound(ErrorIdentificator { fn_call, arg }) => {
                write!(f, "General not found ({}:{})", fn_call, arg)
            }
        }
    }
}

impl Error for GeneralError {}
