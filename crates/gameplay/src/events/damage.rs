use crate::{ActorChange, CorpseChange, TerrainChange};
use text::Subject;

#[derive(Clone, Debug)]
pub(crate) struct Damage {
    // TODO damage types
    pub(crate) attacker: Subject, // for logging
    pub(crate) amount: u16,
}

impl ActorChange for Damage {}
impl CorpseChange for Damage {}
impl TerrainChange for Damage {}
