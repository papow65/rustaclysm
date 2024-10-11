use crate::gameplay::{Damage, Evolution, Healing, Limited};
use bevy::prelude::Component;

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
