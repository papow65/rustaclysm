use bevy::prelude::{Component, Entity, Vec};

/// Used on a tile, for the subzone level that contains the tile.
///
/// Required
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = Tiles)]
pub struct TileIn {
    pub subzone_level_entity: Entity,
}

/// Used on a subzone level, for all tiles in that subzone level.
///
/// Required
#[derive(Debug, Component)]
#[relationship_target(relationship = TileIn, linked_spawn)]
pub struct Tiles {
    object_entities: Vec<Entity>,
}

impl Tiles {
    #[expect(unused)]
    pub(crate) fn object_entities(&self) -> &[Entity] {
        &self.object_entities
    }
}

/// Used on an object, for the tile that contains the object.
///
/// Required
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = Objects)]
pub struct ObjectOn {
    pub tile_entity: Entity,
}

/// Used on a tile, for all objects on the tile.
///
/// Required
#[derive(Debug, Component)]
#[relationship_target(relationship = ObjectOn, linked_spawn)]
pub struct Objects {
    object_entities: Vec<Entity>,
}

impl Objects {
    #[must_use]
    pub fn object_entities(&self) -> &[Entity] {
        &self.object_entities
    }
}
