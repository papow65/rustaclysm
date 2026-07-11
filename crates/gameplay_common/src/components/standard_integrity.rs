use crate::Limited;
use bevy::prelude::Component;

/// Mutable component
#[derive(Debug, PartialEq, Component)]
pub struct StandardIntegrity(pub Limited);

impl StandardIntegrity {
    pub fn lower(&mut self, damage: &crate::Damage) -> crate::Evolution {
        self.0.lower(damage.amount)
    }
    // TODO raising (not with Healing)
}
