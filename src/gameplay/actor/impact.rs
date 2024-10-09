use crate::gameplay::StaminaImpact;
use bevy::prelude::Entity;
use units::Duration;

#[must_use]
#[derive(Debug)]
pub(crate) struct Impact {
    pub(crate) timeout: Duration,
    pub(crate) stamina_impact: StaminaImpact,
}

impl Impact {
    pub(crate) fn check_validity(&self) {
        assert!(Duration::ZERO < self.timeout, "{self:?} is invalid");
    }
}

#[must_use]
#[derive(Debug)]
pub(crate) struct ActorImpact {
    pub(crate) actor_entity: Entity,
    pub(crate) impact: Option<Impact>,
}

impl ActorImpact {
    pub(crate) const fn new(
        actor_entity: Entity,
        timeout: Duration,
        stamina_impact: StaminaImpact,
    ) -> Self {
        Self {
            actor_entity,
            impact: Some(Impact {
                timeout,
                stamina_impact,
            }),
        }
    }

    pub(crate) const fn none(actor_entity: Entity) -> Self {
        Self {
            actor_entity,
            impact: None,
        }
    }

    pub(crate) const fn standing_rest(actor_entity: Entity, timeout: Duration) -> Self {
        Self::new(actor_entity, timeout, StaminaImpact::StandingRest)
    }

    pub(crate) const fn laying_rest(actor_entity: Entity, timeout: Duration) -> Self {
        Self::new(actor_entity, timeout, StaminaImpact::LayingRest)
    }

    pub(crate) const fn heavy(actor_entity: Entity, timeout: Duration) -> Self {
        Self::new(actor_entity, timeout, StaminaImpact::Heavy)
    }

    pub(crate) const fn is_some(&self) -> bool {
        self.impact.is_some()
    }
}
