use crate::prelude::{ActorChange, CorpseChange, Subject, TerrainChange};
use bevy::prelude::Event;

#[derive(Clone, Debug, Event)]
pub(crate) struct Damage {
    // TODO damage types
    pub(crate) attacker: Subject, // for logging
    pub(crate) amount: u16,
}

impl ActorChange for Damage {}
impl CorpseChange for Damage {}
impl TerrainChange for Damage {}

#[derive(Clone, Debug, Event)]
pub(crate) struct Healing {
    pub(crate) amount: u16,
}

impl ActorChange for Healing {}
