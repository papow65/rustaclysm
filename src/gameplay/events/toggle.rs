use crate::gameplay::TerrainChange;
use bevy::prelude::Event;

/// Open or close something, like a door
#[derive(Clone, Debug, PartialEq, Event)]
pub(crate) enum Toggle {
    Open,
    Close,
}

impl TerrainChange for Toggle {}
