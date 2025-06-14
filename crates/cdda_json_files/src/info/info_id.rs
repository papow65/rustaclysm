use bevy_log::warn;
use serde::Deserialize;
use std::hash::{Hash, Hasher};
use std::{any::type_name, fmt, marker::PhantomData, sync::Arc};

use crate::{
    CommonItemInfo, ItemMigration, ItemWithCommonInfo, OvermapTerrainInfo, TerrainInfo,
    VehiclePartInfo, VehiclePartMigration,
};

/// Use [`InfoId`] wherever possible.
/// Note that different info types may use the same ids.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct UntypedInfoId(Arc<str>);

impl UntypedInfoId {
    #[must_use]
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn new_suffix(value: &str, suffix: Option<&str>) -> Self {
        if let Some(suffix) = suffix {
            Self::new(format!("{value}_{suffix}"))
        } else {
            Self::new(value)
        }
    }

    pub fn add_suffix(&mut self, suffix: &str) {
        if suffix.is_empty() {
            warn!("Empty InfoId suffix for {self:?}");
        }
        self.0 = (String::from(&*self.0) + suffix).into();
    }

    #[must_use]
    pub fn starts_with(&self, part: &str) -> bool {
        self.0.starts_with(part)
    }

    #[must_use]
    pub fn contains(&self, part: &str) -> bool {
        self.0.contains(part)
    }

    #[must_use]
    pub fn prefix(&self, part: impl Into<String>) -> Self {
        Self((part.into() + &*self.0).into())
    }

    #[must_use]
    pub fn suffix(&self, part: &str) -> Self {
        Self((String::from(&*self.0) + part).into())
    }

    #[must_use]
    pub fn truncate(&self) -> Self {
        Self(
            String::from(&*self.0)
                .replace("_isolated", "")
                .replace("_end_south", "")
                .replace("_end_west", "")
                .replace("_ne", "")
                .replace("_end_north", "")
                .replace("_ns", "")
                .replace("_es", "")
                .replace("_nes", "")
                .replace("_end_east", "")
                .replace("_wn", "")
                .replace("_ew", "")
                .replace("_new", "")
                .replace("_sw", "")
                .replace("_nsw", "")
                .replace("_esw", "")
                .replace("_nesw", "")
                .into(),
        )
    }

    #[must_use]
    pub fn fallback_name(&self) -> Arc<str> {
        Arc::<str>::from(&*self.0)
    }
}

#[derive(Deserialize)]
#[serde(from = "UntypedInfoId")]
pub struct InfoId<T> {
    untyped: UntypedInfoId,
    _phantom: PhantomData<T>,
}

impl<T> InfoId<T> {
    #[must_use]
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self::from(UntypedInfoId::new(value))
    }

    #[must_use]
    pub fn new_suffix(value: &str, suffix: Option<&str>) -> Self {
        Self::from(UntypedInfoId::new_suffix(value, suffix))
    }

    pub fn add_suffix(&mut self, suffix: &str) {
        self.untyped.add_suffix(suffix);
    }

    #[must_use]
    pub const fn untyped(&self) -> &UntypedInfoId {
        &self.untyped
    }

    #[must_use]
    pub fn fallback_name(&self) -> Arc<str> {
        self.untyped.fallback_name()
    }
}

impl InfoId<OvermapTerrainInfo> {
    #[must_use]
    pub fn is_moving_deep_water_zone(&self) -> bool {
        self.untyped.starts_with("river_")
    }

    #[must_use]
    pub fn is_still_deep_water_zone(&self) -> bool {
        self.untyped.starts_with("lake_")
    }

    #[must_use]
    pub fn is_grassy_zone(&self) -> bool {
        *self == Self::new("field") || self.untyped.starts_with("forest")
    }

    #[must_use]
    pub fn is_road_zone(&self) -> bool {
        self.untyped.starts_with("road_")
    }
}

impl InfoId<TerrainInfo> {
    #[must_use]
    pub fn is_ground(&self) -> bool {
        [Self::new("t_grass"), Self::new("t_dirt")].contains(self)
    }
}

impl<T> Clone for InfoId<T> {
    fn clone(&self) -> Self {
        Self::from(self.untyped.clone())
    }
}

impl<T> fmt::Debug for InfoId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let full_type_name = type_name::<T>();
        let type_short = full_type_name.split("::").last().unwrap_or(full_type_name);
        write!(f, "{type_short} {}", self.untyped.0)
    }
}

impl<T> From<UntypedInfoId> for InfoId<T> {
    fn from(untyped: UntypedInfoId) -> Self {
        Self {
            untyped,
            _phantom: PhantomData,
        }
    }
}

impl<T> Hash for InfoId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.untyped.hash(state);
    }
}

impl<T> PartialEq for InfoId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.untyped == other.untyped
    }
}

impl<T> Eq for InfoId<T> {}

impl<T: ItemWithCommonInfo> From<InfoId<CommonItemInfo>> for InfoId<T> {
    fn from(info_id: InfoId<CommonItemInfo>) -> Self {
        Self::from(info_id.untyped)
    }
}

impl<T: ItemWithCommonInfo> From<InfoId<T>> for InfoId<CommonItemInfo> {
    fn from(info_id: InfoId<T>) -> Self {
        Self::from(info_id.untyped)
    }
}

impl From<InfoId<CommonItemInfo>> for InfoId<ItemMigration> {
    fn from(info_id: InfoId<CommonItemInfo>) -> Self {
        Self::from(info_id.untyped)
    }
}

impl From<InfoId<VehiclePartInfo>> for InfoId<VehiclePartMigration> {
    fn from(info_id: InfoId<VehiclePartInfo>) -> Self {
        Self::from(info_id.untyped)
    }
}

/// Usefull for debugging
#[derive(Deserialize)]
pub struct InfoIdDescription {
    untyped: UntypedInfoId,
    type_name: String,
}

impl fmt::Debug for InfoIdDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.type_name, self.untyped.0)
    }
}

impl<T> From<InfoId<T>> for InfoIdDescription {
    fn from(info_id: InfoId<T>) -> Self {
        let full_type_name = type_name::<T>();
        let type_short = full_type_name.split("::").last().unwrap_or(full_type_name);
        Self {
            untyped: info_id.untyped,
            type_name: String::from(type_short),
        }
    }
}
