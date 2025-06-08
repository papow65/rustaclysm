use crate::{StaminaCost, StaminaImpact};
use bevy::prelude::Entity;
use gameplay_location::Nbor;
use units::Duration;

#[must_use]
#[derive(Debug)]
pub(crate) struct Impact {
    duration: Duration,
    stamina_impact: StaminaImpact,
}

impl Impact {
    pub(crate) fn new(duration: Duration, stamina_impact: StaminaImpact) -> Self {
        assert!(Duration::ZERO < duration, "invalid duration: {duration:?}");

        Self {
            duration,
            stamina_impact,
        }
    }

    pub(crate) const fn duration(&self) -> Duration {
        self.duration
    }

    pub(crate) const fn stamina_impact(&self) -> StaminaImpact {
        self.stamina_impact
    }
}

#[must_use]
#[derive(Debug)]
pub(crate) struct ActorImpact {
    pub(crate) actor_entity: Entity,
    pub(crate) impact: Option<Impact>,
}

impl ActorImpact {
    pub(crate) fn new(
        actor_entity: Entity,
        duration: Duration,
        stamina_impact: StaminaImpact,
    ) -> Self {
        Self {
            actor_entity,
            impact: Some(Impact::new(duration, stamina_impact)),
        }
    }

    pub(crate) const fn none(actor_entity: Entity) -> Self {
        Self {
            actor_entity,
            impact: None,
        }
    }

    pub(crate) fn by_duration(
        actor_entity: Entity,
        duration: Duration,
        cost_per_second: StaminaCost,
    ) -> Self {
        Self::new(
            actor_entity,
            duration,
            StaminaImpact::Duration { cost_per_second },
        )
    }

    pub(crate) fn by_nbor(
        actor_entity: Entity,
        duration: Duration,
        cost_per_meter: StaminaCost,
        nbor: Nbor,
    ) -> Self {
        Self::new(
            actor_entity,
            duration,
            StaminaImpact::Nbor {
                cost_per_meter,
                nbor,
            },
        )
    }

    /// No time passed
    pub(crate) const fn is_some(&self) -> bool {
        self.impact.is_some()
    }
}
