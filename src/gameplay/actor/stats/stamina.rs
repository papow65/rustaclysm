use crate::gameplay::actor::impact::Impact;
use crate::gameplay::{Breath, Limited, Nbor, NborDistance};
use bevy::prelude::Component;
use units::{Distance, Duration};

/// Short term
#[derive(Debug, Component)]
pub(crate) enum Stamina {
    Unlimited,
    Limited(Limited),
}

impl Stamina {
    const MAX: u16 = i16::MAX as u16;

    pub(crate) const FULL: Self = Self::Limited(Limited::full(Self::MAX));

    pub(crate) const fn breath(&self) -> Breath {
        match self {
            Self::Unlimited => Breath::Normal,
            Self::Limited(limited) => {
                let sprint_cost = (-StaminaCost::EXTREME.0) as u16;
                if 2 * sprint_cost <= limited.current() {
                    Breath::Normal
                } else if sprint_cost <= limited.current() {
                    Breath::AlmostWinded
                } else {
                    Breath::Winded
                }
            }
        }
    }

    pub(crate) fn apply(&mut self, impact: &Impact) {
        if let Self::Limited(ref mut limited) = self {
            limited.adjust(impact.stamina_impact().as_i16(impact.duration()));
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum StaminaImpact {
    Duration {
        cost_per_second: StaminaCost,
    },
    Nbor {
        cost_per_meter: StaminaCost,
        nbor: Nbor,
    },
}

impl StaminaImpact {
    fn as_i16(&self, duration: Duration) -> i16 {
        (match self {
            Self::Duration { cost_per_second } => {
                i64::from(cost_per_second.0) * duration.milliseconds() as i64 / 1000
            }
            Self::Nbor {
                cost_per_meter,
                nbor,
            } => {
                i64::from(cost_per_meter.0)
                    * match nbor.distance() {
                        NborDistance::Up => Distance::VERTICAL * 5,
                        NborDistance::Adjacent => Distance::ADJACENT,
                        NborDistance::Diagonal => Distance::DIAGONAL,
                        NborDistance::Zero => panic!("Stamina impact nbor should not be 'here'"),
                        NborDistance::Down => Distance::VERTICAL,
                    }
                    .millimeter() as i64
                    / Distance::ADJACENT.millimeter() as i64
            }
        }) as i16
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct StaminaCost(i16);

impl StaminaCost {
    /// Distance to walk to fully recuperate
    const FULL_RECUPERATION: u16 = 1000; // (adjacent) tiles
    const WALK_GAIN: i16 = (Stamina::MAX / Self::FULL_RECUPERATION) as i16;

    /// Distance to sprint to be fully out of breath
    const OUT_OF_BREATH: u16 = 50; // (adjacent) tiles
    const SPRINT_COST: i16 = (Stamina::MAX / Self::OUT_OF_BREATH) as i16;

    /// For example when lying down
    pub(crate) const LYING_REST: Self = Self(Self::WALK_GAIN * 4);
    /// For example when sitting still
    #[expect(unused)]
    pub(crate) const SITTING_REST: Self = Self(Self::WALK_GAIN * 3);
    /// For example when standing still
    pub(crate) const STANDING_REST: Self = Self(Self::WALK_GAIN * 2);
    /// For example when walking
    pub(crate) const LIGHT: Self = Self(Self::WALK_GAIN);
    /// For example when speed walking
    pub(crate) const NEUTRAL: Self = Self(0);
    /// For example when running
    pub(crate) const HEAVY: Self = Self(-Self::SPRINT_COST / 2);
    /// For example when sprinting
    pub(crate) const EXTREME: Self = Self(-Self::SPRINT_COST);
}
