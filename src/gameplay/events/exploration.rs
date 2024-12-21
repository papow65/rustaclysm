use crate::gameplay::{Pos, ZoneLevel};
use bevy::prelude::Event;

#[derive(Debug, Event)]
pub(crate) enum Exploration {
    /// The player has seen the pos
    Pos(Pos),
    /// The player has seen the zone level on the map
    ZoneLevel(ZoneLevel),
}
