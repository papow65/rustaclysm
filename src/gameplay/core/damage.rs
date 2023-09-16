use crate::prelude::{ActorChange, Fragment, ItemChange};
use bevy::prelude::Event;

#[derive(Clone, Debug, Event)]
pub(crate) struct Damage {
    // TODO damage types
    pub(crate) attacker: Fragment, // for logging
    pub(crate) amount: u16,
}

impl ActorChange for Damage {}
impl ItemChange for Damage {}

#[derive(Clone, Debug, Event)]
pub(crate) struct Healing {
    pub(crate) amount: u16,
}

impl ActorChange for Healing {}
