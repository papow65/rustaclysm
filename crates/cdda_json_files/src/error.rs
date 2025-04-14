use crate::InfoIdDescription;
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum Error {
    UnknownInfoId { _id: InfoIdDescription },
    UnknownExamineAction { _str: String },
    LinkUnavailable { _id: InfoIdDescription },
}

// Requirement for StdError
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl StdError for Error {}
