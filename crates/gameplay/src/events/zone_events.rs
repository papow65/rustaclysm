use crate::{SubzoneLevel, ZoneLevel};
use bevy::prelude::{Entity, Event};

#[derive(Debug, Event)]
pub(crate) struct SpawnSubzoneLevel {
    pub(crate) subzone_level: SubzoneLevel,
}

#[derive(Debug, Event)]
pub(crate) struct DespawnSubzoneLevel {
    pub(crate) subzone_level: SubzoneLevel,
}

#[derive(Debug, Event)]
pub(crate) struct SpawnZoneLevel {
    pub(crate) zone_level: ZoneLevel,
}

#[derive(Debug, Event)]
pub(crate) struct UpdateZoneLevelVisibility {
    pub(crate) zone_level: ZoneLevel,
    pub(crate) children: Vec<Entity>,
}

#[derive(Debug, Event)]
pub(crate) struct DespawnZoneLevel {
    pub(crate) entity: Entity,
}
