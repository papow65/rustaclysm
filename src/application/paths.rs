use crate::prelude::{LoadError, SavPath};
use bevy::ecs::system::Resource;
use glob::glob;
use std::{
    any::type_name,
    fmt,
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub(crate) struct PathFor<T>(pub(crate) PathBuf, PhantomData<T>);

impl<T> PathFor<T> {
    pub(crate) fn init(path: PathBuf) -> PathFor<T> {
        PathFor(path, PhantomData)
    }
}

impl<T> fmt::Debug for PathFor<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        "PathFor<".fmt(formatter)?;
        type_name::<T>().fmt(formatter)?;
        ">{".fmt(formatter)?;
        self.0.fmt(formatter)?;
        "}".fmt(formatter)
    }
}

pub(crate) type WorldPath = PathFor<()>;

/// This represents a world and a save in that world
#[derive(Clone, Resource)]
pub(crate) struct Paths {
    sav_path: PathBuf,
}

impl Paths {
    fn asset_path() -> &'static Path {
        Path::new("assets")
    }

    pub(crate) fn data_path() -> PathBuf {
        Self::asset_path().join("data")
    }

    pub(crate) fn fonts_path() -> PathBuf {
        Self::asset_path().join("fonts")
    }

    pub(crate) fn gfx_path() -> PathBuf {
        Self::asset_path().join("gfx")
    }

    fn save_path() -> PathBuf {
        Self::asset_path().join("save")
    }

    pub(crate) fn new(path: &PathBuf) -> Self {
        let sav_path = Self::save_path().join(path);
        //println!("Loading {}...", sav_path.display());
        Self { sav_path }
    }

    pub(crate) fn sav_path(&self) -> SavPath {
        SavPath::init(self.sav_path.clone())
    }

    pub(crate) fn tiles_path() -> PathBuf {
        Self::asset_path().join("tiles")
    }

    pub(crate) fn world_path(&self) -> WorldPath {
        WorldPath::init(
            self.sav_path
                .parent()
                .expect("Parent directory expected")
                .to_path_buf(),
        )
    }

    pub(crate) fn list() -> Result<Vec<PathBuf>, LoadError> {
        Self::check_dirs()?;

        let worlds_pattern = Self::save_path().join("*");
        let pattern = worlds_pattern.to_str().unwrap();
        let worlds = glob(pattern)
            .expect("Failed to read glob pattern")
            .map(|world| {
                world
                    .expect("problem with path")
                    .components()
                    .skip(2)
                    .collect::<PathBuf>()
            })
            .collect::<Vec<_>>();

        if worlds.is_empty() {
            Err(LoadError::new(vec![
                format!(
                    "No Cataclysm: DDA worlds found to load under {}",
                    Self::save_path().display()
                ),
                String::from("Create a new world using Cataclysm: DDA to continue."),
            ]))
        } else {
            let savs_pattern = worlds_pattern.join("#*.sav");
            let pattern = savs_pattern.to_str().unwrap();
            let savs = glob(pattern)
                .expect("Failed to read glob pattern")
                .map(|sav| {
                    sav.expect("problem with path")
                        .components()
                        .skip(2)
                        .collect::<PathBuf>()
                })
                .collect::<Vec<_>>();

            if savs.is_empty() {
                Err(LoadError::new(vec![
                    format!(
                        "No Cataclysm: DDA saves found to load in any world directory under {}",
                        Self::save_path().display()
                    ),
                    String::from("Create a new save file using Cataclysm: DDA to continue."),
                ]))
            } else {
                Ok(savs)
            }
        }
    }

    fn check_dirs() -> Result<(), LoadError> {
        if !Self::asset_path().is_dir() {
            return Err(LoadError::new(vec![
                format!("Directory '{}' not found.", Self::asset_path().display()),
                String::from("Please run this in the directory containing the 'assets' directory."),
            ]));
        }

        for asset_subdir in vec![Self::data_path(), Self::gfx_path(), Self::save_path()] {
            if !asset_subdir.is_dir() {
                return Err(LoadError::new(vec![
                    format!("Directory '{}/' not found.", asset_subdir.display()),
                        format!("Please make sure the '{}/' directory contains a copy of (or a symlink to) Cataclysm-DDA's '{}/' directory.", Self::asset_path().display(), asset_subdir.file_name().unwrap().to_str().unwrap())
                ]));
            }
        }

        Ok(())
    }
}
