use crate::TerrainChange;

/// Open or close something, like a door
#[derive(Clone, Debug, PartialEq)]
pub enum Toggle {
    Open,
    Close,
}

impl TerrainChange for Toggle {}
