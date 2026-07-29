use std::{error::Error as StdError, fmt, path::PathBuf, sync::Arc};

/// This file lacks a non-JSON first line
#[derive(Debug)]
pub struct MissingFirstLine {
    pub(crate) _path: PathBuf,
    pub(crate) _contents: Arc<str>,
}

// Requirement for StdError
impl fmt::Display for MissingFirstLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl StdError for MissingFirstLine {}
