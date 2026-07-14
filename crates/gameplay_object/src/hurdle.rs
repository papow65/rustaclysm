use bevy::prelude::Component;
use cdda_json_files::MoveCostIncrease;

/// Slows movement
#[derive(Component)]
#[component(immutable)]
pub struct Hurdle(pub MoveCostIncrease);
