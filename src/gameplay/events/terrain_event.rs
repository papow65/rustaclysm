use bevy::prelude::{Entity, Event};

pub(crate) trait TerrainChange: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug, Event)]
pub(crate) struct TerrainEvent<T: TerrainChange> {
    pub(crate) terrain_entity: Entity,
    pub(crate) change: T,
}
