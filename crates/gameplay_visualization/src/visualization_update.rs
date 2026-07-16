use bevy::prelude::Resource;

/// Strategy to use when updating visualizations
#[derive(Debug, Default, PartialEq, Eq, Resource)]
pub enum VisualizationUpdate {
    #[default]
    Smart,
    Forced,
}

impl VisualizationUpdate {
    #[must_use]
    pub fn forced(&self) -> bool {
        *self == Self::Forced
    }

    pub const fn reset(&mut self) {
        *self = Self::Smart;
    }
}
