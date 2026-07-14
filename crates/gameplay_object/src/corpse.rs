use bevy::prelude::Component;
use units::Timestamp;

#[derive(Component)]
#[component(immutable)]
pub struct Corpse;

#[derive(Component)]
#[component(immutable)]
pub struct CorpseRaise {
    pub at: Timestamp,
}
