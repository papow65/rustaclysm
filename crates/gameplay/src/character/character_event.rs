use bevy::prelude::{Entity, Message};
use gameplay_common::{Damage, Healing};

pub(crate) trait CharacterChange: Clone + Send + Sync + 'static {}

impl CharacterChange for Damage {}
impl CharacterChange for Healing {}

#[must_use]
#[derive(Clone, Debug, Message)]
pub(crate) struct CharacterEvent<A: CharacterChange> {
    pub(crate) actor_entity: Entity,
    pub(crate) action: A,
}

impl<A: CharacterChange> CharacterEvent<A> {
    pub(crate) const fn new(actor_entity: Entity, action: A) -> Self {
        Self {
            actor_entity,
            action,
        }
    }
}
