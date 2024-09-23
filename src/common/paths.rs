use bevy::ecs::system::Resource;
use cdda::Sav;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::{any::type_name, fmt, marker::PhantomData};

pub(crate) struct PathFor<T>(pub(crate) PathBuf, PhantomData<T>);

impl<T> PathFor<T> {
    pub(crate) const fn init(path: PathBuf) -> Self {
        Self(path, PhantomData)
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

pub(crate) type SavPath = PathFor<Sav>;

impl TryFrom<&SavPath> for Sav {
    type Error = serde_json::Error;

    fn try_from(sav_path: &SavPath) -> Result<Self, Self::Error> {
        read_to_string(&sav_path.0)
            .ok()
            .inspect(|_| {
                println!("Loading {}...", sav_path.0.display());
            })
            .map(|s| String::from(s.split_at(s.find('\n').expect("Non-JSON first line")).1))
            .map(|s| serde_json::from_str::<Self>(s.as_str()))
            .expect(".sav file could not be read")
    }
}
//

/// This represents a world and a save in that world
#[derive(Clone, Resource)]
pub(crate) struct Paths {
    sav_path: PathBuf,
}

impl Paths {
    pub(crate) fn asset_path() -> &'static Path {
        Path::new("assets")
    }

    pub(crate) fn backgrounds_path() -> PathBuf {
        Self::asset_path().join("backgrounds")
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

    pub(crate) fn save_path() -> PathBuf {
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

    pub(crate) fn world_path(&self) -> WorldPath {
        WorldPath::init(
            self.sav_path
                .parent()
                .expect("Path should have a parent directory")
                .to_path_buf(),
        )
    }
}
