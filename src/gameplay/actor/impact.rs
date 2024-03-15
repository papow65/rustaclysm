use crate::prelude::{Milliseconds, StaminaImpact};
use bevy::prelude::Entity;

#[must_use]
#[derive(Debug)]
pub(crate) struct Impact {
    pub(crate) actor_entity: Entity,
    pub(crate) timeout: Milliseconds,
    pub(crate) stamina_impact: Option<StaminaImpact>,
}

impl Impact {
    pub(crate) const fn new(
        actor_entity: Entity,
        timeout: Milliseconds,
        stamina_impact: Option<StaminaImpact>,
    ) -> Self {
        Self {
            actor_entity,
            timeout,
            stamina_impact,
        }
    }

    pub(crate) const fn none(actor_entity: Entity) -> Self {
        Self::new(actor_entity, Milliseconds::ZERO, None)
    }

    pub(crate) const fn rest(actor_entity: Entity, timeout: Milliseconds) -> Self {
        Self::new(actor_entity, timeout, Some(StaminaImpact::Rest))
    }

    pub(crate) const fn full_rest(actor_entity: Entity, timeout: Milliseconds) -> Self {
        Self::new(actor_entity, timeout, Some(StaminaImpact::FullRest))
    }

    pub(crate) const fn heavy(actor_entity: Entity, timeout: Milliseconds) -> Self {
        Self::new(actor_entity, timeout, Some(StaminaImpact::Heavy))
    }

    pub(crate) fn check_validity(&self) {
        assert_eq!(
            self.timeout == Milliseconds::ZERO,
            self.stamina_impact.is_none(),
            "{self:?} is invalid"
        );
    }
}
