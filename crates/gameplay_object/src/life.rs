use bevy::prelude::Component;

/// To be removed on death, to be added when revived
#[derive(Debug, Component)]
#[component(immutable)]
pub struct Life;
