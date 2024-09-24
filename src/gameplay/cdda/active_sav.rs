use crate::common::AssetPaths;
use crate::gameplay::cdda::{SavPath, WorldPath};
use bevy::ecs::system::Resource;
use std::path::PathBuf;

/// This represents a world and a save in that world.
///
/// See also [`AssetPaths`](`crate::common::AssetPaths`)
#[derive(Clone, Resource)]
pub(crate) struct ActiveSav {
    sav_path: PathBuf,
}

impl ActiveSav {
    pub(crate) fn new(path: &PathBuf) -> Self {
        let sav_path = AssetPaths::save().join(path);
        //println!("Loading {}...", sav_path.display());
        Self { sav_path }
    }

    pub(crate) fn sav_path(&self) -> SavPath {
        SavPath::init(self.sav_path.clone())
    }

    pub(crate) fn world_path(&self) -> WorldPath {
        WorldPath::init(
            self.sav_path
                .parent()
                .expect("Path to sav file should have a parent directory")
                .to_path_buf(),
        )
    }
}
