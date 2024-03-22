use crate::prelude::Timestamp;
use bevy::prelude::{Entity, Resource};

#[derive(Resource)]
pub(super) struct CraftingScreen {
    pub(super) panel: Entity,
    pub(super) last_time: Timestamp,
}
