use crate::gameplay::cdda::TypeId;
use cdda_json_files::{ObjectId, SpriteNumber};
use std::fmt::{self, Display};
use std::{error::Error as StdError, io, path::PathBuf, sync::Arc};

#[derive(Debug)]
pub(crate) enum Error {
    /// This recipe has no time set, where it is expected
    RecipeWithoutTime {
        _id: ObjectId,
    },
    /// This id, or combination of id and type is not known
    UnknownObject {
        _id: ObjectId,
        _type: &'static [TypeId],
    },
    /// This id doesn't match with items or requirments
    UnexpectedRequirement {
        _id: ObjectId,
    },
    /// This sprite numer is not known
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

// Requirement for StdError
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
