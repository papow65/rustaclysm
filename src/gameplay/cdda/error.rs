use crate::gameplay::cdda::TypeId;
use cdda_json_files::{Error as CddaJsonError, ObjectId, SpriteNumber};
use std::{error::Error as StdError, fmt, io, path::PathBuf, sync::Arc};

#[derive(Debug)]
pub(crate) enum Error {
    /// This recipe has no time set, where it is expected
    RecipeWithoutTime {
        _id: ObjectId,
    },
    /// This id, or combination of id and type is not known
    UnknownObject {
        _id: ObjectId,
        _type: &'static str,
    },
    /// This id doesn't match with items or requirments
    UnexpectedRequirement {
        _id: ObjectId,
    },
    /// This sprite numer is not known
    UnknownSpriteNumber {
        _number: SpriteNumber,
    },
    /// This [`TypeId`] is not known
    UnknownTypeId {
        _type: TypeId,
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
