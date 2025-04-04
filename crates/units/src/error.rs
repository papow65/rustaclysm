use std::num::{ParseFloatError, ParseIntError};
use std::{error::Error as StdError, fmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknowUnit { _value: String },
    UnknowFloat { _wrapped: ParseFloatError },
    UnknowInteger { _wrapped: ParseIntError },
}

// Requirement for StdError
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Self::UnknowFloat { _wrapped: value }
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::UnknowInteger { _wrapped: value }
    }
}

impl StdError for Error {}
