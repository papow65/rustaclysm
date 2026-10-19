use bevy::prelude::Resource;
use gameplay_perception::Region;

/// The region whose subzone levels should be expanded into tiles.
#[derive(Default, Resource)]
pub struct Expanded {
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
