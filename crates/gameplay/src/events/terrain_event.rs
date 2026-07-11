use crate::Toggle;
use bevy::prelude::{Entity, Message};
use gameplay_common::Damage;

pub(crate) trait TerrainChange: Clone + Send + Sync + 'static {}

impl TerrainChange for Damage {}
impl TerrainChange for Toggle {}

#[must_use]
#[derive(Clone, Debug, Message)]
pub(crate) struct TerrainEvent<C: TerrainChange> {
    /// Terrain or furniture
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
