use bevy::prelude::{Entity, Message};
use gameplay_common::Damage;

pub trait CorpseChange: Clone + Send + Sync + 'static {}

impl CorpseChange for Damage {}

#[must_use]
#[derive(Clone, Debug, Message)]
pub struct CorpseEvent<C: CorpseChange> {
    pub corpse_entity: Entity,
    pub change: C,
}

impl<C: CorpseChange> CorpseEvent<C> {
    pub const fn new(corpse_entity: Entity, change: C) -> Self {
        Self {
            corpse_entity,
            change,
        }
    }
}
