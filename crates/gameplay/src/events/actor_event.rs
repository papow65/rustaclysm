use bevy::prelude::{Entity, Message};

pub(crate) trait ActorChange: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug, Message)]
pub(crate) struct ActorEvent<A: ActorChange> {
    pub(crate) actor_entity: Entity,
    pub(crate) action: A,
}

impl<A: ActorChange> ActorEvent<A> {
    pub(crate) const fn new(actor_entity: Entity, action: A) -> Self {
        Self {
            actor_entity,
            action,
        }
    }
}
