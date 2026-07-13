use bevy::prelude::Component;
use cdda_json_files::CddaItem;
use gameplay_location::PosOffset;
use std::sync::Arc;

/// Vehicle component
#[derive(Component)]
#[component(immutable)]
pub struct Vehicle;

/// Vehicle part component
#[derive(Component)]
#[component(immutable)]
pub struct VehiclePart {
    pub offset: PosOffset,
    pub item: Arc<CddaItem>,
}
