mod plugin;

pub use plugin::GameplayRefreshPlugin;

use bevy::prelude::Resource;

/// Scope to use when refreshing the gameplay world.
#[derive(Debug, Default, PartialEq, Eq, Resource)]
pub enum RefreshScope {
    #[default]
    NearPlayer,
    AllLoaded,
}

impl RefreshScope {
    #[must_use]
    pub fn is_all_loaded(&self) -> bool {
        *self == Self::AllLoaded
    }

    pub const fn reset(&mut self) {
        *self = Self::NearPlayer;
    }
}
