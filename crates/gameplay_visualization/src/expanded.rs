use bevy::prelude::Resource;
use gameplay_common::Region;

/// The area that should be expanded into tiles
#[derive(Default, Resource)]
pub struct Expanded {
    pub region: Region,
}

impl Expanded {
    pub fn update(&mut self, region: Region) -> bool {
        let changed = self.region != region;
        if changed {
            self.region = region;
        }
        changed
    }
}
