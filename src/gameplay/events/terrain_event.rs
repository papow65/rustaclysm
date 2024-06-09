use bevy::prelude::{Entity, Event};

pub(crate) trait TerrainChange: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug, Event)]
pub(crate) struct TerrainEvent<C: TerrainChange> {
    /// Terraain or furniture
    pub(crate) terrain_entity: Entity,
    pub(crate) change: C,
}

impl<C: TerrainChange> TerrainEvent<C> {
    pub(crate) const fn new(terrain_entity: Entity, change: C) -> Self {
        Self {
            terrain_entity,
            change,
        }
    }
}
