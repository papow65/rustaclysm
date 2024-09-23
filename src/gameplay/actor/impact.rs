use crate::gameplay::StaminaImpact;
use bevy::prelude::Entity;
use units::Duration;

#[must_use]
#[derive(Debug)]
pub(crate) struct Impact {
    pub(crate) actor_entity: Entity,
    pub(crate) timeout: Duration,
    pub(crate) stamina_impact: Option<StaminaImpact>,
}

impl Impact {
    pub(crate) const fn new(
        actor_entity: Entity,
        timeout: Duration,
        stamina_impact: Option<StaminaImpact>,
    ) -> Self {
        Self {
            actor_entity,
            timeout,
            stamina_impact,
        }
    }

    pub(crate) const fn standing_rest(actor_entity: Entity, timeout: Duration) -> Self {
        Self::new(actor_entity, timeout, Some(StaminaImpact::StandingRest))
    }

    pub(crate) const fn laying_rest(actor_entity: Entity, timeout: Duration) -> Self {
        Self::new(actor_entity, timeout, Some(StaminaImpact::LayingRest))
    }

    pub(crate) const fn heavy(actor_entity: Entity, timeout: Duration) -> Self {
        Self::new(actor_entity, timeout, Some(StaminaImpact::Heavy))
    }

    pub(crate) fn check_validity(&self) {
        assert_eq!(
            self.timeout == Duration::ZERO,
            self.stamina_impact.is_none(),
            "{self:?} is invalid"
        );
    }
}
