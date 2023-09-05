use bevy::prelude::{Entity, Resource};

#[derive(Resource)]
pub(crate) struct DeathScreen {
    pub(super) root: Entity,
}
