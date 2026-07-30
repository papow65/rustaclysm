use bevy::ecs::change_detection::Tick;
use bevy::prelude::{FromWorld, Resource, World};
use std::ops::Deref;

#[derive(Debug, Resource)]
pub(crate) struct GameplayStart(Tick);

impl Deref for GameplayStart {
    type Target = Tick;

    fn deref(&self) -> &Tick {
        &self.0
    }
}

impl FromWorld for GameplayStart {
    fn from_world(world: &mut World) -> Self {
        Self(world.change_tick())
    }
}
