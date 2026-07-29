use crate::DynInfoId;
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum Error {
    UnknownInfoId { _id: DynInfoId },
    UnknownExamineAction { _str: String },
    LinkUnavailable { _id: DynInfoId },
}

// Requirement for StdError
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl StdError for Error {}
