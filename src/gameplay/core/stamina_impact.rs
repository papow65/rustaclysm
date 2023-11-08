use crate::prelude::Action;

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

impl Action for StaminaImpact {}
