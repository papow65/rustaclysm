use crate::ObjectId;
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum Error {
    /// This id, or combination of id and type is not known
    LinkUnavailable { _id: ObjectId, _type: &'static str },
}

// Requirement for StdError
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl StdError for Error {}
