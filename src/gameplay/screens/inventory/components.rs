use crate::gameplay::Phrase;
use bevy::prelude::{Component, Entity};
use std::fmt;

#[derive(Clone, Debug)]
pub(super) enum InventoryAction {
    Examine,
    Take,
    Drop,
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

#[derive(Debug, Component)]
pub(super) struct InventoryItemRow {
    pub(super) item: Entity,
}

#[derive(Debug, Component)]
pub(super) struct InventoryItemDescription(pub(super) Phrase);
