use bevy::prelude::{Component, Entity, Vec};

/// Used on a tile, for the subzone level that contains the tile.
///
/// Required
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = Tiles)]
pub(crate) struct TileIn {
    pub(crate) subzone_level_entity: Entity,
}

/// Used on a subzone level, for all tiles in that subzone level.
///
/// Required
#[derive(Debug, Component)]
#[relationship_target(relationship = TileIn, linked_spawn)]
pub(crate) struct Tiles {
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
pub(crate) struct ObjectOn {
    pub(crate) tile_entity: Entity,
}

/// Used on a tile, for all objects on the tile.
///
/// Required
#[derive(Debug, Component)]
#[relationship_target(relationship = ObjectOn, linked_spawn)]
pub(crate) struct Objects {
    object_entities: Vec<Entity>,
}

impl Objects {
    pub(crate) fn object_entities(&self) -> &[Entity] {
        &self.object_entities
    }
}

/// Used on a vehicle part, for the vehicle that contains the vehicle part.
///
/// Required
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = VehicleParts)]
pub(crate) struct VehiclePartOf {
    pub(crate) vehicle_entity: Entity,
}

/// Used on a vehicle, for all vehicle parts of the vehicle.
///
/// Required
#[derive(Debug, Component)]
#[relationship_target(relationship = VehiclePartOf)]
pub(crate) struct VehicleParts {
    vehicle_part_entities: Vec<Entity>,
}

impl VehicleParts {
    #[expect(unused)]
    pub(crate) fn vehicle_part_entities(&self) -> &[Entity] {
        &self.vehicle_part_entities
    }
}
