use crate::prelude::*;
use bevy::prelude::Component;
use std::fmt;

// Stats for characters
// See player.rs for player-only stats

#[derive(Component, Clone, Copy)]
pub(crate) struct BaseSpeed(MillimeterPerSecond);

impl BaseSpeed {
    pub(crate) const fn from_percent(percent: u64) -> Self {
        Self::from_kmph(percent / 10)
    }

    pub(crate) const fn from_kmph(s: u64) -> Self {
        Self(MillimeterPerSecond::from_kmph(s))
    }

    pub(crate) fn npc_speed(&self) -> MillimeterPerSecond {
        self.0
    }

    pub(crate) fn player_speed(
        &self,
        stamina: &Stamina,
        walking_mode: &WalkingMode,
    ) -> MillimeterPerSecond {
        MillimeterPerSecond(
            (self.0 .0 as f32
                * if stamina.can_run() {
                    walking_mode.speed_factor()
                } else {
                    WalkingMode::Staggering.speed_factor()
                }) as u64,
        )
    }
}

impl fmt::Display for BaseSpeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Max speed {}", self.0)
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
