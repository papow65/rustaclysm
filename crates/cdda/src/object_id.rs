use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct ObjectId(Arc<str>);

impl ObjectId {
    pub fn new(value: &str) -> Self {
        Self(value.into())
    }

    pub fn starts_with(&self, part: &str) -> bool {
        self.0.starts_with(part)
    }

    pub fn contains(&self, part: &str) -> bool {
        self.0.contains(part)
    }

    pub fn prefix(&self, part: impl Into<String>) -> Self {
        Self((part.into() + &*self.0).into())
    }

    pub fn suffix(&self, part: &str) -> Self {
        Self((String::from(&*self.0) + part).into())
    }

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

    pub fn fallback_name(&self) -> String {
        String::from(&*self.0)
    }

    pub fn is_moving_deep_water_zone(&self) -> bool {
        self.0.starts_with("river_")
    }

    pub fn is_still_deep_water_zone(&self) -> bool {
        self.0.starts_with("lake_")
    }

    pub fn is_grassy_zone(&self) -> bool {
        &*self.0 == "field" || self.0.starts_with("forest")
    }

    pub fn is_road_zone(&self) -> bool {
        self.0.starts_with("road_")
    }

    pub fn is_ground(&self) -> bool {
        &*self.0 == "t_grass" || &*self.0 == "t_dirt"
    }
}
