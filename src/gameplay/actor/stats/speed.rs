use crate::gameplay::{Breath, StaminaCost};
use crate::hud::{BAD_TEXT_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, WARN_TEXT_COLOR};
use bevy::prelude::{Component, Mix, TextColor};
use units::Speed;

// Stats for characters

#[derive(Debug, Clone, Copy, Component)]
pub(crate) struct BaseSpeed(Speed);

impl BaseSpeed {
    pub(crate) fn from_percent(percent: u64) -> Self {
        Self::from_kmph(percent as f32 * 0.05) // 100 % -> 5 km/h
    }

    pub(crate) fn from_kmph(s: f32) -> Self {
        Self(Speed::from_kmph(s))
    }

    pub(crate) fn speed(&self, walking_mode: &WalkingMode, breath: Breath) -> Speed {
        let average_speed = WalkingMode::Walking.standard_speed(Breath::Normal);
        self.0
            .combine(walking_mode.standard_speed(breath), average_speed)
    }
}

#[derive(Debug, Component)]
pub(crate) enum WalkingMode {
    Crouching,
    Walking,
    SpeedWalking,
    Running,
    Sprinting,
}

impl WalkingMode {
    /// Various factors may adjuct this speed
    #[must_use]
    pub(crate) fn standard_speed(&self, breath: Breath) -> Speed {
        Speed::from_kmph(match breath {
            Breath::Normal | Breath::AlmostWinded => match self {
                Self::Crouching => 2.0,
                Self::Walking => 5.0,
                Self::SpeedWalking => 6.5, // just below 7.0, the speed of zombies
                Self::Running => 10.0,
                Self::Sprinting => 15.0,
            },
            Breath::Winded => 1.0,
        })
    }

    #[must_use]
    pub(crate) const fn stamina_impact(&self, breath: Breath) -> StaminaCost {
        match breath {
            Breath::Normal | Breath::AlmostWinded => match self {
                Self::Walking => StaminaCost::LIGHT,
                Self::Crouching | Self::SpeedWalking => StaminaCost::NEUTRAL,
                Self::Running => StaminaCost::HEAVY,
                Self::Sprinting => StaminaCost::EXTREME,
            },
            Breath::Winded => StaminaCost::LIGHT,
        }
    }

    #[must_use]
    pub(crate) const fn switch(&self) -> Self {
        match self {
            Self::Crouching => Self::Walking,
            Self::Walking => Self::SpeedWalking,
            Self::SpeedWalking => Self::Running,
            Self::Running => Self::Sprinting,
            Self::Sprinting => Self::Crouching,
        }
    }

    #[must_use]
    pub(crate) const fn as_str(&self) -> &str {
        match self {
            Self::Crouching => "Crouching",
            Self::Walking => "Walking",
            Self::SpeedWalking => "Speed walking",
            Self::Running => "Running",
            Self::Sprinting => "Sprinting",
        }
    }

    #[must_use]
    pub(crate) fn breath_color(&self) -> TextColor {
        match self {
            Self::Walking => GOOD_TEXT_COLOR,
            Self::Crouching | Self::SpeedWalking => {
                TextColor(HARD_TEXT_COLOR.0.mix(&WARN_TEXT_COLOR.0, 0.5))
            }
            Self::Running => WARN_TEXT_COLOR,
            Self::Sprinting => BAD_TEXT_COLOR,
        }
    }
}
