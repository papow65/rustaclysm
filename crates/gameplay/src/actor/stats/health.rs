use crate::{Damage, Healing};
use bevy::prelude::Component;
use gameplay_common::{Evolution, Limited};

/// Not to be removed on death, because it is needed again when revived
///
/// Mutable component
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
