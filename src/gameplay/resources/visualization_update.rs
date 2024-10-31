use bevy::prelude::Resource;

/// Strategy to use when updating visualizations
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Resource)]
pub(crate) enum VisualizationUpdate {
    #[default]
    Smart,
    Forced,
}

impl VisualizationUpdate {
    pub(crate) fn forced(&self) -> bool {
        *self == Self::Forced
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::Smart;
    }
}
