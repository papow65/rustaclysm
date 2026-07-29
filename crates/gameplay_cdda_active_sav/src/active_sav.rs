use crate::MissingFirstLine;
use anyhow::{Context as _, Error as AnyError};
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
    pub fn new(path: &PathBuf) -> Result<Self, AnyError> {
        let sav_path = AssetPaths::save().join(path);
        //trace!("Loading {}...", sav_path.display());

        let sav_contents = read_to_string(&sav_path)?;
        debug!("Loading {}...", sav_path.display());
        let sav_contents = String::from(
            sav_contents
                .split_at(sav_contents.find('\n').ok_or_else(|| MissingFirstLine {
                    _path: sav_path.clone(),
                    _contents: sav_contents.as_str().into(),
                })?)
                .1,
        );
        let sav = from_json_str::<Sav>(&sav_contents).with_context(|| {
            format!(
                "Failed loading {}\nFile contents: {sav_contents}",
                path.display()
            )
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
