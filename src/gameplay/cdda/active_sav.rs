use crate::common::AssetPaths;
use crate::gameplay::cdda::{SavPath, WorldPath};
use bevy::ecs::system::Resource;
use cdda_json_files::Sav;
use std::{fs::read_to_string, path::PathBuf};

/// This represents a world and a save in that world.
///
/// See also [`AssetPaths`](`crate::common::AssetPaths`)
#[derive(Resource)]
pub(crate) struct ActiveSav {
    sav_path: PathBuf,
    sav: Sav,
}

impl ActiveSav {
    pub(crate) fn new(path: &PathBuf) -> Self {
        let sav_path = AssetPaths::save().join(path);
        //println!("Loading {}...", sav_path.display());

        let sav = read_to_string(&sav_path)
            .ok()
            .inspect(|_| {
                println!("Loading {}...", sav_path.display());
            })
            .map(|s| String::from(s.split_at(s.find('\n').expect("Non-JSON first line")).1))
            .map(|s| serde_json::from_str::<Sav>(s.as_str()))
            .expect(".sav file could not be read")
            .expect("Loading sav file failed");
        Self { sav_path, sav }
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

    pub(crate) const fn sav(&self) -> &Sav {
        &self.sav
    }
}
