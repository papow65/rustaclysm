use crate::prelude::{Action, Subject, TerrainChange};
use bevy::prelude::Event;

#[derive(Clone, Debug, Event)]
pub(crate) struct Damage {
    // TODO damage types
    pub(crate) attacker: Subject, // for logging
    pub(crate) amount: u16,
}

impl Action for Damage {}
impl TerrainChange for Damage {}

#[derive(Clone, Debug, Event)]
pub(crate) struct Healing {
    pub(crate) amount: u16,
}

impl Action for Healing {}
