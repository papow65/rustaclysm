use bevy::prelude::Component;
use cdda_json_files::MoveCost;

/// Terrain that can be accessed, like a floor
#[derive(Component)]
#[component(immutable)]
pub struct Accessible {
    pub water: bool,
    pub move_cost: MoveCost,
}

/// Blocks vision to and from the level below
#[derive(Component)]
#[component(immutable)]
pub struct OpaqueFloor;
