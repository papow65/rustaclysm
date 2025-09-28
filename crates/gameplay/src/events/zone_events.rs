use bevy::prelude::{Entity, Message};
use gameplay_location::{SubzoneLevel, ZoneLevel};

#[derive(Debug, Message)]
pub(crate) struct SpawnSubzoneLevel {
    pub(crate) subzone_level: SubzoneLevel,
}

#[derive(Debug, Message)]
pub(crate) struct DespawnSubzoneLevel {
    pub(crate) subzone_level: SubzoneLevel,
}

#[derive(Debug, Message)]
pub(crate) struct SpawnZoneLevel {
    pub(crate) zone_level: ZoneLevel,
}

#[derive(Debug, Message)]
pub(crate) struct UpdateZoneLevelVisibility {
    pub(crate) zone_level: ZoneLevel,
    pub(crate) children: Vec<Entity>,
}

#[derive(Debug, Message)]
pub(crate) struct DespawnZoneLevel {
    pub(crate) entity: Entity,
}
