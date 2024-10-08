use crate::gameplay::{Breath, Damage, Evolution, Healing, Limited, StaminaImpact};
use crate::hud::{HARD_TEXT_COLOR, WARN_TEXT_COLOR};
use bevy::prelude::{Color, Component};
use std::fmt;
use units::Speed;

// Stats for characters

#[derive(Debug, Clone, Copy, Component)]
pub(crate) struct BaseSpeed(Speed);

impl BaseSpeed {
    pub(crate) const fn from_percent(percent: u64) -> Self {
        Self::from_kmph(percent / 10)
    }

    pub(crate) const fn from_kmph(s: u64) -> Self {
        Self(Speed::from_kmph(s))
    }

    pub(crate) fn speed(&self, walking_mode: &WalkingMode, breath: Breath) -> Speed {
        self.0 * walking_mode.speed_factor(breath)
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
    pub(crate) const fn speed_factor(&self, breath: Breath) -> f32 {
        match breath {
            Breath::Normal | Breath::AlmostWinded => match self {
                Self::Crouching => 0.25,
                Self::Walking => 0.5,
                Self::SpeedWalking => 0.65,
                Self::Running => 1.0,
            },
            Breath::Winded => 0.1,
        }
    }

    #[must_use]
    pub(crate) const fn stamina_impact(&self, breath: Breath) -> StaminaImpact {
        match breath {
            Breath::Normal | Breath::AlmostWinded => match self {
                Self::Crouching | Self::Walking => StaminaImpact::Light,
                Self::SpeedWalking => StaminaImpact::Neutral,
                Self::Running => StaminaImpact::Heavy,
            },
            Breath::Winded => StaminaImpact::Light,
        }
    }

    #[must_use]
    pub(crate) const fn switch(&self) -> Self {
        match self {
            Self::Crouching => Self::Walking,
            Self::Walking => Self::SpeedWalking,
            Self::SpeedWalking => Self::Running,
            Self::Running => Self::Crouching,
        }
    }

    #[must_use]
    pub(crate) const fn as_str(&self) -> &str {
        match self {
            Self::Crouching => "Crouching",
            Self::Walking => "Walking",
            Self::SpeedWalking => "Speed walking",
            Self::Running => "Running",
        }
    }

    #[must_use]
    pub(crate) const fn color(&self) -> Color {
        match self {
            Self::Walking => HARD_TEXT_COLOR,
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
    pub(crate) const fn breath(&self) -> Breath {
        match self {
            Self::Unlimited => Breath::Normal,
            Self::Limited(limited) => {
                let run_cost = -StaminaImpact::Heavy.as_i16() as u16;
                if 2 * run_cost <= limited.current() {
                    Breath::Normal
                } else if run_cost <= limited.current() {
                    Breath::AlmostWinded
                } else {
                    Breath::Winded
                }
            }
        }
    }

    pub(crate) fn apply(&mut self, stamina_impact: StaminaImpact) {
        if let Self::Limited(ref mut limited) = self {
            limited.adjust(stamina_impact.as_i16());
        }
    }
}

#[derive(Debug, Component)]
pub(crate) struct Health(pub(crate) Limited);

impl Health {
    pub(crate) fn lower(&mut self, damage: &Damage) -> Evolution {
        self.0.lower(damage.amount)
    }

    pub(crate) fn raise(&mut self, healing: &Healing) -> Evolution {
        self.0.raise(healing.amount)
    }
}

/// For some animals
#[derive(Debug, Component)]
pub(crate) struct Aquatic;
