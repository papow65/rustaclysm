use bevy::prelude::Component;

/// Used for objects that can move, like characters and vehicles
#[derive(Debug, Component)]
#[component(immutable)]
pub struct Mobile;
