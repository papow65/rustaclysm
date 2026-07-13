use bevy::prelude::{Component, Entity, Vec};

/// Used on a vehicle part, for the vehicle that contains the vehicle part.
///
/// Required
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = VehicleParts)]
pub struct VehiclePartOf {
    pub vehicle_entity: Entity,
}

/// Used on a vehicle, for all vehicle parts of the vehicle.
///
/// Required
#[derive(Debug, Component)]
#[relationship_target(relationship = VehiclePartOf, linked_spawn)]
pub struct VehicleParts {
    vehicle_part_entities: Vec<Entity>,
}

impl VehicleParts {
    #[must_use]
    pub fn vehicle_part_entities(&self) -> &[Entity] {
        &self.vehicle_part_entities
    }
}
