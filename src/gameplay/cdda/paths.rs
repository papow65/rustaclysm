use crate::gameplay::{Overzone, ZoneLevel};
use cdda_json_files::{Map, MapMemory, Overmap, OvermapBuffer, Sav};
use std::{any::type_name, fmt, fs::read_to_string, marker::PhantomData, path::PathBuf};

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

pub(super) type MapPath = PathFor<Map>;

impl MapPath {
    pub(super) fn new(world_path: &WorldPath, zone_level: ZoneLevel) -> Self {
        Self::init(
            world_path
                .0
                .join("maps")
                .join(format!(
                    "{}.{}.{}",
                    zone_level.zone.x.div_euclid(32),
                    zone_level.zone.z.div_euclid(32),
                    zone_level.level.h,
                ))
                .join(format!(
                    "{}.{}.{}.map",
                    zone_level.zone.x, zone_level.zone.z, zone_level.level.h
                )),
        )
    }
}

pub(super) type MapMemoryPath = PathFor<MapMemory>;

impl MapMemoryPath {
    pub(super) fn new(sav_path: &SavPath, zone_level: ZoneLevel) -> Self {
        let mut seen_path = sav_path.0.clone();
        seen_path.set_extension("mm1");
        let seen_path = seen_path.join(format!(
            "{}.{}.{}.mmr",
            zone_level.zone.x.div_euclid(4),
            zone_level.zone.z.div_euclid(4),
            zone_level.level.h
        ));
        Self::init(seen_path)
    }
}

pub(super) type OvermapPath = PathFor<Overmap>;

impl OvermapPath {
    pub(crate) fn new(world_path: &WorldPath, overzone: Overzone) -> Self {
        Self::init(
            world_path
                .0
                .join(format!("o.{}.{}", overzone.x, overzone.z)),
        )
    }
}

pub(super) type OvermapBufferPath = PathFor<OvermapBuffer>;

impl OvermapBufferPath {
    pub(crate) fn new(sav_path: &SavPath, overzone: Overzone) -> Self {
        let mut seen_path = sav_path.0.clone();
        seen_path.set_extension(format!("seen.{}.{}", overzone.x, overzone.z));
        Self::init(seen_path)
    }
}
