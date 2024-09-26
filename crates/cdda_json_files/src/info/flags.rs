use crate::structure::MaybeFlatVec;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Default, Deserialize)]
#[serde(from = "Option<MaybeFlatVec<Arc<str>>>")]
#[serde(default)]
pub struct Flags(Vec<Arc<str>>);

impl Flags {
    fn contains(&self, value: &str) -> bool {
        self.0.contains(&Arc::from(value))
    }

    pub fn aquatic(&self) -> bool {
        self.contains("AQUATIC")
    }

    pub fn goes_up(&self) -> bool {
        self.contains("GOES_UP") || self.contains("RAMP_UP")
    }

    pub fn goes_down(&self) -> bool {
        self.contains("GOES_DOWN") || self.contains("RAMP_DOWN")
    }

    pub fn obstacle(&self) -> bool {
        self.contains("OBSTACLE")
    }

    pub fn transparent(&self) -> bool {
        self.contains("TRANSPARENT")
    }

    pub fn water(&self) -> bool {
        self.contains("SHALLOW_WATER") || self.contains("DEEP_WATER")
    }
}

impl From<Option<MaybeFlatVec<Arc<str>>>> for Flags {
    fn from(cdda_flags: Option<MaybeFlatVec<Arc<str>>>) -> Self {
        Self(match cdda_flags {
            Some(MaybeFlatVec(flags)) => flags,
            None => Vec::new(),
        })
    }
}