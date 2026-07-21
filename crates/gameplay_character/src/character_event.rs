use bevy::prelude::{Entity, Message};
use gameplay_object::{Damage, Healing};

pub trait CharacterChange: Clone + Send + Sync + 'static {}

impl CharacterChange for Damage {}
impl CharacterChange for Healing {}

#[must_use]
#[derive(Clone, Debug, Message)]
pub struct CharacterEvent<A: CharacterChange> {
    pub actor_entity: Entity,
    pub action: A,
}

impl<A: CharacterChange> CharacterEvent<A> {
    pub const fn new(actor_entity: Entity, action: A) -> Self {
        Self {
            actor_entity,
            action,
        }
    }
}
