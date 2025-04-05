use std::num::{ParseFloatError, ParseIntError};
use std::{error::Error as StdError, fmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknowUnit { _value: String },
    ExpectedFloat { _wrapped: ParseFloatError },
    ExpectedInteger { _wrapped: ParseIntError },
}

// Requirement for StdError
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Self::ExpectedFloat { _wrapped: value }
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::ExpectedInteger { _wrapped: value }
    }
}

impl StdError for Error {}
