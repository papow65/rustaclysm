use bevy::prelude::Component;

/// Used to indicate a root for all non-mobile objects with the same position
#[derive(Debug, Component)]
#[component(immutable)]
pub struct Tile;
