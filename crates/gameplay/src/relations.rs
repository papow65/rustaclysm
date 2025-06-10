use bevy::prelude::{Component, Entity, Vec};

/// Used on an object, for the subzone level that contains the object.
///
/// Required
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = Objects)]
pub(crate) struct ObjectIn {
    pub(crate) subzone_level_entity: Entity,
}

/// Used on a subzone level, for all objects that the subzone level has.
///
/// Required
#[derive(Debug, Component)]
#[relationship_target(relationship = ObjectIn, linked_spawn)]
pub(crate) struct Objects {
    object_entities: Vec<Entity>,
}

impl Objects {
    #[expect(unused)]
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
