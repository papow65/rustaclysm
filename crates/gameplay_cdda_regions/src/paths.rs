use crate::{MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset};
use cdda_json_files::Sav;
use gameplay_location::{Overzone, ZoneLevel};
use std::{any::type_name, fmt, marker::PhantomData, path::PathBuf};

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

pub(super) type MapPath = PathFor<MapAsset>;

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

pub(super) type MapMemoryPath = PathFor<MapMemoryAsset>;

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

pub(super) type OvermapPath = PathFor<OvermapAsset>;

impl OvermapPath {
    pub(crate) fn new(world_path: &WorldPath, overzone: Overzone) -> Self {
        Self::init(
            world_path
                .0
                .join(format!("o.{}.{}", overzone.x, overzone.z)),
        )
    }
}

pub(super) type OvermapBufferPath = PathFor<OvermapBufferAsset>;

impl OvermapBufferPath {
    pub(crate) fn new(sav_path: &SavPath, overzone: Overzone) -> Self {
        let mut seen_path = sav_path.0.clone();
        seen_path.set_extension(format!("seen.{}.{}", overzone.x, overzone.z));
        Self::init(seen_path)
    }
}
