use crate::Error;
use bevy::prelude::{Resource, debug};
use cdda_json_files::Sav;
use serde_json::from_str as from_json_str;
use std::{fs::read_to_string, path::PathBuf};
use util::AssetPaths;

/// This represents a world and a save in that world.
///
/// See also [`AssetPaths`](`util::AssetPaths`)
#[derive(Resource)]
pub struct ActiveSav {
    sav_path: PathBuf,
    sav: Sav,
}

impl ActiveSav {
    /// # Errors
    /// - When the file could not be read
    /// - When the first non-JSON line is missing
    /// - When the rest of the file could not be parsed
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let sav_path = AssetPaths::save().join(path);
        //trace!("Loading {}...", sav_path.display());

        let sav = read_to_string(&sav_path)
            .map_err(|err| Error::Io { _wrapped: err })
            .inspect(|_| {
                debug!("Loading {}...", sav_path.display());
            })
            .and_then(|s: String| {
                Ok(String::from(
                    s.split_at(s.find('\n').ok_or_else(|| Error::MissingFirstLine {
                        _path: sav_path.clone(),
                        _contents: s.as_str().into(),
                    })?)
                    .1,
                ))
            })
            .and_then(|s| {
                from_json_str::<Sav>(s.as_str()).map_err(|err| Error::JsonWithContext {
                    _wrapped: err,
                    _file_path: path.clone(),
                    _contents: s.as_str().into(),
                })
            })?;
        Ok(Self { sav_path, sav })
    }

    #[must_use]
    pub const fn path(&self) -> &PathBuf {
        &self.sav_path
    }

    #[must_use]
    pub const fn sav(&self) -> &Sav {
        &self.sav
    }
}
