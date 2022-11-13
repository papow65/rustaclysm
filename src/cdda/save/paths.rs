use crate::prelude::SavPath;
use bevy::ecs::system::Resource;
use glob::glob;
use std::{
    any::type_name,
    env::args,
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

    fn gfx_path() -> PathBuf {
        Self::asset_path().join("gfx")
    }

    fn save_path() -> PathBuf {
        Self::asset_path().join("save")
    }

    fn new(path: &PathBuf) -> Self {
        let sav_path = Self::save_path().join(path);
        println!("Loading {}...", sav_path.display());
        Self { sav_path }
    }

    pub(crate) fn sav_path(&self) -> SavPath {
        SavPath::init(self.sav_path.clone())
    }

    pub(crate) fn world_path(&self) -> WorldPath {
        WorldPath::init(
            self.sav_path
                .parent()
                .expect("Parent directory expected")
                .to_path_buf(),
        )
    }

    /// Load only matching file, load from command line argument, or show error with explanation
    pub(crate) fn load() -> Result<Self, ()> {
        if !Self::asset_path().is_dir() {
            eprintln!("Directory '{}' not found.", Self::asset_path().display());
            eprintln!("Please run this in the directory containing the 'assets' directory.");
            return Err(());
        }

        for asset_subdir in vec![Self::data_path(), Self::gfx_path(), Self::save_path()] {
            if !asset_subdir.is_dir() {
                eprintln!("Directory '{}/' not found.", asset_subdir.display());
                eprintln!("Please make sure the '{}/' directory contains a copy of (or a symlink to) Cataclysm-DDA's '{}/' directory.", Self::asset_path().display(), asset_subdir.file_name().unwrap().to_str().unwrap());
                return Err(());
            }
        }

        let args: Vec<String> = args().collect();
        if args.len() == 2 && !args[0].starts_with('-') {
            Ok(Paths::new(&PathBuf::from(args[1].as_str())))
        } else {
            eprintln!("No (or too many) arguments speciefied");
            let mut possibilites: Vec<PathBuf> = Vec::new();
            let pattern = Self::save_path().join("**").join("#*.sav");
            let pattern = pattern.to_str().unwrap();
            for sav in glob(pattern).expect("Failed to read glob pattern") {
                possibilites.push(
                    sav.expect("problem with path")
                        .components()
                        .skip(2)
                        .collect::<PathBuf>(),
                );
            }
            match possibilites.len() {
                0 => {
                    let example = Self::save_path().join("Boston").join("#VGFsZG9y.sav");
                    eprintln!("A file like '{}' should exist. Create a world and a character save using Cataclysm-DDA.", example.display());
                    Err(())
                }
                1 => {
                    println!("Found a single save files: {}", possibilites[0].display());
                    Ok(Paths::new(&possibilites[0]))
                }
                n => {
                    eprintln!(
                        "Found {n} save files under {}:",
                        Self::save_path().display()
                    );
                    for possibility in possibilites {
                        eprintln!("'{}'", possibility.display());
                    }
                    eprintln!("Please specify the one of the .sav files above to load as command line argument.");
                    eprintln!("Include quotes to prevent '#...' being interpreted as a comment.");
                    Err(())
                }
            }
        }
    }
}
