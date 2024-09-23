use crate::SpriteNumber;
use std::fmt::{self, Display};
use std::{error::Error as StdError, io, path::PathBuf, sync::Arc};

#[derive(Debug)]
pub enum Error {
    UnknownSpriteNumber {
        _number: SpriteNumber,
    },

    // External error wrappers
    Io {
        _wrapped: io::Error,
    },
    Json {
        _wrapped: serde_json::Error,
        _file_path: PathBuf,
        _contents: Arc<str>,
    },
}

impl Display for Error {
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
