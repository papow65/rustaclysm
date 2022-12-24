mod cdda_furniture_info;
mod cdda_item_info;
mod cdda_overmap_info;
mod cdda_terrain_info;

pub(crate) use self::{
    cdda_furniture_info::*, cdda_item_info::*, cdda_overmap_info::*, cdda_terrain_info::*,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Flags(Vec<String>);

impl Flags {
    fn contains(&self, value: &str) -> bool {
        self.0.contains(&String::from(value))
    }

    pub(crate) fn goes_up(&self) -> bool {
        self.contains("GOES_UP") || self.contains("RAMP_UP") || self.contains("CLIMBABLE")
    }

    pub(crate) fn goes_down(&self) -> bool {
        self.contains("GOES_DOWN") || self.contains("RAMP_DOWN") || self.contains("CLIMBABLE")
    }

    pub(crate) fn is_transparent(&self) -> bool {
        self.contains("TRANSPARENT")
    }
}
