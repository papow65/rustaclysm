use std::{error::Error as StdError, fmt, io, path::PathBuf, sync::Arc};

#[derive(Debug)]
pub enum Error {
    /// This file lacks a non-JSON first line
    MissingFirstLine {
        _path: PathBuf,
        _contents: Arc<str>,
    },

    // External error wrappers
    Io {
        _wrapped: io::Error,
    },
    JsonWithContext {
        _wrapped: serde_json::Error,
        _file_path: PathBuf,
        _contents: Arc<str>,
    },
}

// Requirement for StdError
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io { _wrapped: value }
    }
}

impl StdError for Error {}
