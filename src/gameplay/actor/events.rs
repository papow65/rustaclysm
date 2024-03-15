use bevy::prelude::{Entity, Event};

pub(crate) trait ActorChange: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug, Event)]
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

#[derive(Clone, Debug, Event)]
pub(crate) struct Healing {
    pub(crate) amount: u16,
}

impl ActorChange for Healing {}

#[derive(Copy, Clone, Debug)]
pub(crate) enum StaminaImpact {
    FullRest,
    Rest,
    Light,
    Neutral,
    Heavy,
}

impl StaminaImpact {
    pub(crate) const fn as_i16(&self) -> i16 {
        match self {
            Self::FullRest => 100,
            Self::Rest => 2,
            Self::Light => 1,
            Self::Neutral => 0,
            Self::Heavy => -12,
        }
    }
}

impl ActorChange for StaminaImpact {}
