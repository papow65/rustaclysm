use crate::prelude::*;
use bevy::prelude::{Color, Component};
use std::fmt;

// Stats for characters

#[derive(Component, Clone, Copy)]
pub(crate) struct BaseSpeed(MillimeterPerSecond);

impl BaseSpeed {
    pub(crate) const fn from_percent(percent: u64) -> Self {
        Self::from_kmph(percent / 10)
    }

    pub(crate) const fn from_kmph(s: u64) -> Self {
        Self(MillimeterPerSecond::from_kmph(s))
    }

    pub(crate) fn speed(&self, walking_mode: &WalkingMode, breath: Breath) -> MillimeterPerSecond {
        MillimeterPerSecond((self.0 .0 as f32 * walking_mode.speed_factor(breath)) as u64)
    }
}

impl fmt::Display for BaseSpeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Max speed {}", self.0)
    }
}

#[derive(Debug, Component)]
pub(crate) enum WalkingMode {
    Crouching,
    Walking,
    SpeedWalking,
    Running,
}

impl WalkingMode {
    #[must_use]
    pub(crate) fn speed_factor(&self, breath: Breath) -> f32 {
        match breath {
            Breath::Normal => match self {
                Self::Crouching => 0.25,
                Self::Walking => 0.5,
                Self::SpeedWalking => 0.65,
                Self::Running => 1.0,
            },
            Breath::Winded => 0.1,
        }
    }

    #[must_use]
    pub(crate) fn stamina_impact(&self, breath: Breath) -> StaminaImpact {
        match breath {
            Breath::Normal => match self {
                Self::Crouching | Self::Walking => StaminaImpact::Light,
                Self::SpeedWalking => StaminaImpact::Neutral,
                Self::Running => StaminaImpact::Heavy,
            },
            Breath::Winded => StaminaImpact::Light,
        }
    }

    #[must_use]
    pub(crate) fn switch(&self) -> Self {
        match self {
            Self::Crouching => Self::Walking,
            Self::Walking => Self::SpeedWalking,
            Self::SpeedWalking => Self::Running,
            Self::Running => Self::Crouching,
        }
    }

    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Crouching => "Crouching",
            Self::Walking => "Walking",
            Self::SpeedWalking => "Speed walking",
            Self::Running => "Running",
        }
    }

    #[must_use]
    pub(crate) fn color(&self) -> Color {
        match self {
            Self::Walking => DEFAULT_TEXT_COLOR,
            Self::Crouching | Self::SpeedWalking | Self::Running => WARN_TEXT_COLOR,
        }
    }
}

#[derive(Debug, Component)]
pub(crate) enum Stamina {
    Unlimited,
    Limited(Limited),
}

impl Stamina {
    pub(crate) fn breath(&self) -> Breath {
        match self {
            Self::Unlimited => Breath::Normal,
            Self::Limited(limited) => {
                let run_cost = -StaminaImpact::Heavy.as_i16() as u16;
                if run_cost <= limited.current() {
                    Breath::Normal
                } else {
                    Breath::Winded
                }
            }
        }
    }

    pub(crate) fn apply(&mut self, stamina_impact: StaminaImpact) {
        if let Self::Limited(ref mut limited) = self {
            limited.saturating_add(stamina_impact.as_i16());
        }
    }
}

#[derive(Debug, Component)]
pub(crate) struct Health(pub(crate) Limited);

impl Health {
    pub(crate) fn apply(&mut self, damage: &Damage) -> bool {
        self.0.saturating_subtract(damage.amount);
        self.0.is_nonzero()
    }
}

/** For some animals */
#[derive(Component)]
pub(crate) struct Aquatic;
