use crate::gameplay::events::ActorChange;

#[derive(Copy, Clone, Debug)]
pub(crate) enum StaminaImpact {
    LayingRest,
    StandingRest,
    Light,
    Neutral,
    Heavy,
}

impl StaminaImpact {
    pub(crate) const fn as_i16(&self) -> i16 {
        match self {
            Self::LayingRest => 4,
            Self::StandingRest => 2,
            Self::Light => 1,
            Self::Neutral => 0,
            Self::Heavy => -12,
        }
    }
}

impl ActorChange for StaminaImpact {}
