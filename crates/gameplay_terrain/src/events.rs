use bevy::prelude::{Entity, Message};
use gameplay_common::Damage;

pub trait TerrainChange: Clone + Send + Sync + 'static {}

impl TerrainChange for Damage {}

#[must_use]
#[derive(Clone, Debug, Message)]
pub struct TerrainEvent<C: TerrainChange> {
    /// Terrain or furniture
    pub terrain_entity: Entity,
    pub change: C,
}

impl<C: TerrainChange> TerrainEvent<C> {
    pub const fn new(terrain_entity: Entity, change: C) -> Self {
        Self {
            terrain_entity,
            change,
        }
    }
}
