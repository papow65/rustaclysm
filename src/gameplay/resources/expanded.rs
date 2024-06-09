use crate::prelude::Region;
use bevy::prelude::Resource;

/// The area that should be expanded into tiles
#[derive(Default, Resource)]
pub(crate) struct Expanded {
    pub(crate) region: Region,
}

impl Expanded {
    pub(crate) fn update(&mut self, region: Region) -> bool {
        let changed = self.region != region;
        if changed {
            self.region = region;
        }
        changed
    }
}
