use cdda_json_files::{Error as CddaJsonError, InfoId, InfoIdDescription, Recipe, SpriteNumber};
use std::{error::Error as StdError, fmt, io, path::PathBuf, sync::Arc};

#[derive(Debug)]
pub enum Error {
    /// This recipe has no time set, where it is expected
    RecipeWithoutTime {
        _id: InfoId<Recipe>,
    },
    /// This id, or combination of id and type is not known
    UnknownObject {
        _id: InfoIdDescription,
    },
    /// This sprite numer is not known
    UnknownSpriteNumber {
        _number: SpriteNumber,
    },

    // Workspace error wrappers
    CddaJsonFiles {
        _wrapped: CddaJsonError,
    },

    // External error wrappers
    Io {
        _wrapped: io::Error,
    },
    Json {
        _wrapped: serde_json::Error,
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

impl From<CddaJsonError> for Error {
    fn from(value: CddaJsonError) -> Self {
        Self::CddaJsonFiles { _wrapped: value }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io { _wrapped: value }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json { _wrapped: value }
    }
}

impl StdError for Error {}
