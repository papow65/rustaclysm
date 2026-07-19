use bevy::prelude::Component;
use gameplay_common::{Damage, Healing};
use gameplay_common::{Evolution, Limited};
use units::Duration;

/// Not to be removed on death, because it is needed again when revived
///
/// Mutable component
#[derive(Debug, Component)]
pub struct Health(Limited);

impl Health {
    pub fn full(max: u16) -> Self {
        Self(Limited::full(max))
    }
    
    pub fn lower(&mut self, damage: &Damage) -> Evolution {
        self.0.lower(damage.amount)
    }

    pub fn raise(&mut self, healing: &Healing) -> Evolution {
        self.0.raise(healing.amount)
    }

    pub fn value(&self) -> &Limited {
        &self.0
    }
}

/// How long an actor has been healing
///
/// Mutable component
#[derive(Debug, Component)]
pub struct HealingDuration(Duration);

impl HealingDuration {
    pub const fn new() -> Self {
        Self(Duration::ZERO)
    }

    #[must_use]
    pub fn heal(&mut self, duration: Duration) -> u64 {
        let healing_rate = Duration::SECOND * 1000;

        self.0 += duration;
        self.0.extract_div(healing_rate)
    }
}
