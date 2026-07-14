use bevy::prelude::Component;
use gameplay_common::{Damage, Evolution, Limited};

/// Mutable component
#[derive(Debug, PartialEq, Component)]
pub struct StandardIntegrity(pub Limited);

impl StandardIntegrity {
    pub fn lower(&mut self, damage: &Damage) -> Evolution {
        self.0.lower(damage.amount)
    }
    // TODO raising (not with Healing)
}
