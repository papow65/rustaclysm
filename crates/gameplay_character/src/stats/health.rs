use bevy::prelude::Component;
use gameplay_common::{Evolution, Limited};
use gameplay_object::{Damage, Healing};
use units::Duration;

/// Not to be removed on death, because it is needed again when revived
///
/// Mutable component
#[must_use]
#[derive(Debug, Component)]
pub struct Health(Limited);

impl Health {
    pub const fn full(max: u16) -> Self {
        Self(Limited::full(max))
    }

    pub fn lower(&mut self, damage: &Damage) -> Evolution {
        self.0.lower(damage.amount)
    }

    pub fn raise(&mut self, healing: &Healing) -> Evolution {
        self.0.raise(healing.amount)
    }

    pub const fn value(&self) -> &Limited {
        &self.0
    }
}

/// How long an actor has been healing
///
/// Mutable component
#[derive(Debug, Component)]
pub struct HealingDuration(Duration);

impl HealingDuration {
    #[must_use]
    pub fn heal(&mut self, duration: Duration) -> u64 {
        let healing_rate = Duration::SECOND * 1000;

        self.0 += duration;
        self.0.extract_div(healing_rate)
    }
}

impl Default for HealingDuration {
    fn default() -> Self {
        Self(Duration::ZERO)
    }
}
