use crate::prelude::{Actor, ActorItem};
use bevy::prelude::{Entity, Event, Query};

pub(crate) trait ActorChange: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug, Event)]
pub(crate) struct ActorEvent<T: ActorChange> {
    pub(crate) actor_entity: Entity,
    pub(crate) change: T,
}

impl<T: ActorChange> ActorEvent<T> {
    pub(crate) const fn new(actor_entity: Entity, change: T) -> Self {
        Self {
            actor_entity,
            change,
        }
    }

    pub(crate) fn actor<'a>(&self, actors: &'a Query<Actor>) -> ActorItem<'a> {
        actors.get(self.actor_entity).expect("Actor entity")
    }
}
