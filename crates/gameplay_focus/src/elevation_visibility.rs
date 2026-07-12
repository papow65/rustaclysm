use bevy::prelude::Resource;

/// Visibility of tiles above the player character
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Resource)]
pub enum ElevationVisibility {
    #[default]
    Shown,
    Hidden,
}
