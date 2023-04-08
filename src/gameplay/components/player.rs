use crate::prelude::{Limited, StaminaImpact};
use bevy::prelude::Component;

#[derive(Component)]
pub(crate) struct Player;

/** Only for the player */
#[derive(Debug, Component)]
pub(crate) enum WalkingMode {
    Staggering,
    Walking,
    Running,
}

impl WalkingMode {
    #[must_use]
    pub(crate) fn speed_factor(&self) -> f32 {
        match self {
            Self::Staggering => 0.1,
            Self::Walking => 0.5,
            Self::Running => 1.0,
        }
    }

    #[must_use]
    pub(crate) fn stamina_impact(&self) -> StaminaImpact {
        match self {
            Self::Staggering => StaminaImpact::Rest,
            Self::Walking => StaminaImpact::Light,
            Self::Running => StaminaImpact::Heavy,
        }
    }

    #[must_use]
    pub(crate) fn switch(&self) -> Self {
        match self {
            Self::Staggering => panic!(),
            Self::Walking => Self::Running,
            Self::Running => Self::Walking,
        }
    }
}

/** Only for the player */
#[derive(Debug, Component)]
pub(crate) struct Stamina(pub(crate) Limited);

impl Stamina {
    pub(crate) fn can_run(&self) -> bool {
        let run_cost = -StaminaImpact::Heavy.as_i16() as u16;
        run_cost < self.0.current()
    }

    pub(crate) fn apply(&mut self, stamina_impact: StaminaImpact) {
        self.0.saturating_add(stamina_impact.as_i16());
    }
}
