mod character_info;
mod furniture_info;
mod item_info;
mod migration;
mod overmap_info;
mod terrain_info;

pub(crate) use self::{
    character_info::*, furniture_info::*, item_info::*, migration::*, overmap_info::*,
    terrain_info::*,
};

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Flags(Vec<String>);

impl Flags {
    fn contains(&self, value: &str) -> bool {
        self.0.contains(&String::from(value))
    }

    pub(crate) fn aquatic(&self) -> bool {
        self.contains("AQUATIC")
    }

    pub(crate) fn goes_up(&self) -> bool {
        self.contains("GOES_UP") || self.contains("RAMP_UP") || self.contains("CLIMBABLE")
    }

    pub(crate) fn goes_down(&self) -> bool {
        self.contains("GOES_DOWN") || self.contains("RAMP_DOWN") || self.contains("CLIMBABLE")
    }

    pub(crate) fn transparent(&self) -> bool {
        self.contains("TRANSPARENT")
    }

    pub(crate) fn water(&self) -> bool {
        self.contains("SHALLOW_WATER") || self.contains("DEEP_WATER")
    }
}
