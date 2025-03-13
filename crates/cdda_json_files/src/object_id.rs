use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub struct ObjectId(Arc<str>);

impl ObjectId {
    #[must_use]
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn new_suffix(value: &String, suffix: Option<&str>) -> Self {
        if let Some(suffix) = suffix {
            Self::new(value.clone() + suffix)
        } else {
            Self::new(value.as_str())
        }
    }

    pub fn add_suffix(&mut self, suffix: &str) {
        assert!(!suffix.is_empty());
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

    #[must_use]
    pub fn is_moving_deep_water_zone(&self) -> bool {
        self.0.starts_with("river_")
    }

    #[must_use]
    pub fn is_still_deep_water_zone(&self) -> bool {
        self.0.starts_with("lake_")
    }

    #[must_use]
    pub fn is_grassy_zone(&self) -> bool {
        &*self.0 == "field" || self.0.starts_with("forest")
    }

    #[must_use]
    pub fn is_road_zone(&self) -> bool {
        self.0.starts_with("road_")
    }

    #[must_use]
    pub fn is_ground(&self) -> bool {
        &*self.0 == "t_grass" || &*self.0 == "t_dirt"
    }
}
