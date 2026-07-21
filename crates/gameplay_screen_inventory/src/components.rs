use bevy::prelude::{Component, Entity};
use keyboard::Key;
use std::fmt;
use strum::VariantArray;

#[derive(Clone, Copy, Debug, PartialEq, VariantArray)]
pub(super) enum InventoryAction {
    Examine,
    Take,

    /// Move and Drop have the same effect.
    Drop,
    /// Move and Drop have the same effect.
    Move,

    Wield,
    Unwield,
}

impl fmt::Display for InventoryAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Examine => write!(f, "Examine"),
            Self::Take => write!(f, "Take"),
            Self::Drop => write!(f, "Drop"),
            Self::Move => write!(f, "Move"),
            Self::Wield => write!(f, "Wield"),
            Self::Unwield => write!(f, "Unwield"),
        }
    }
}

impl From<InventoryAction> for Key {
    fn from(value: InventoryAction) -> Self {
        Self::Character(match value {
            InventoryAction::Examine => 'e',
            InventoryAction::Take => 't',
            InventoryAction::Drop => 'd',
            InventoryAction::Move => 'm',
            InventoryAction::Wield => 'w',
            InventoryAction::Unwield => 'u',
        })
    }
}

#[derive(Debug, Component)]
#[component(immutable)]
pub(super) struct InventoryItemRow {
    pub(super) item: Entity,
}
