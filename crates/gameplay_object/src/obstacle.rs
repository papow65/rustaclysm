use bevy::prelude::Component;

/// Not accessible for any movement
#[derive(Component)]
#[component(immutable)]
pub struct Obstacle;
