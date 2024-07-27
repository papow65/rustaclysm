use crate::prelude::SpriteNumber;
use std::{io, path::PathBuf};

#[derive(Debug)]
pub(crate) enum Error {
    MissingJsonKey {
        _format: &'static str,
        _key: &'static str,
        _json: serde_json::Value,
    },
    UnexpectedJsonVariant {
        _format: &'static str,
        _part: Option<&'static str>,
        _expected: &'static str,
        _json: serde_json::Value,
    },
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
        _contents: String,
    },
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io { _wrapped: value }
    }
}
