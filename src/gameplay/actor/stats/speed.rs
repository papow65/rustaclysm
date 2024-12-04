use crate::gameplay::{Breath, ChangePace, StaminaCost};
use crate::hud::{BAD_TEXT_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, WARN_TEXT_COLOR};
use bevy::prelude::{Component, Mix as _, TextColor};
use units::Speed;

#[derive(Debug, Clone, Copy, Component)]
pub(crate) struct BaseSpeed(Speed);

impl BaseSpeed {
    pub(crate) fn from_percent(percent: u64) -> Self {
        Self::from_kmph(percent as f32 * 0.05) // 100 % -> 5 km/h
    }

    pub(crate) const fn from_kmph(s: f32) -> Self {
        Self(Speed::from_kmph(s))
    }

    pub(crate) const fn speed(&self, walking_mode: &WalkingMode, breath: Breath) -> Speed {
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
    /// Used for enemies (without stamina)
    Perpetual,
}

impl WalkingMode {
    /// Various factors may adjuct this speed
    #[must_use]
    pub(crate) const fn standard_speed(&self, breath: Breath) -> Speed {
        Speed::from_kmph(match breath {
            Breath::Normal | Breath::AlmostWinded => match self {
                Self::Crouching => 2.0,
                Self::Walking => 5.0,
                Self::SpeedWalking => 6.5,
                Self::Running => 10.0,
                Self::Sprinting => 15.0,
                Self::Perpetual => 12.3,
            },
            Breath::Winded => 1.0,
        })
    }

    #[must_use]
    pub(crate) const fn stamina_impact(&self, breath: Breath) -> StaminaCost {
        match breath {
            Breath::Normal | Breath::AlmostWinded => match self {
                Self::Walking => StaminaCost::LIGHT,
                Self::Crouching | Self::SpeedWalking | Self::Perpetual => StaminaCost::NEUTRAL,
                Self::Running => StaminaCost::HEAVY,
                Self::Sprinting => StaminaCost::EXTREME,
            },
            Breath::Winded => StaminaCost::LIGHT,
        }
    }

    #[must_use]
    pub(crate) const fn switch(&self, change_pace: ChangePace) -> Self {
        match change_pace {
            ChangePace::Next => match self {
                Self::Crouching => Self::Walking,
                Self::Walking => Self::SpeedWalking,
                Self::SpeedWalking => Self::Running,
                Self::Running => Self::Sprinting,
                Self::Sprinting => Self::Crouching,
                Self::Perpetual => Self::Perpetual,
            },
            ChangePace::Previous => match self {
                Self::Crouching => Self::Sprinting,
                Self::Walking => Self::Crouching,
                Self::SpeedWalking => Self::Walking,
                Self::Running => Self::SpeedWalking,
                Self::Sprinting => Self::Running,
                Self::Perpetual => Self::Perpetual,
            },
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
            Self::Perpetual => "Perpetual speed",
        }
    }

    #[must_use]
    pub(crate) fn breath_color(&self) -> TextColor {
        match self {
            Self::Walking | Self::Perpetual => GOOD_TEXT_COLOR,
            Self::Crouching | Self::SpeedWalking => {
                TextColor(HARD_TEXT_COLOR.0.mix(&WARN_TEXT_COLOR.0, 0.5))
            }
            Self::Running => WARN_TEXT_COLOR,
            Self::Sprinting => BAD_TEXT_COLOR,
        }
    }
}
