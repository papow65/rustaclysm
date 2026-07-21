use bevy::prelude::Component;

/// Mutable component
#[derive(Clone, PartialEq, Eq, Debug, Component)]
pub enum LastSeen {
    Currently,
    Previously, // TODO consider adding a timestamp
    Never,
}
