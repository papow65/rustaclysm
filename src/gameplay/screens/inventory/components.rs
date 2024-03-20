use bevy::prelude::{Component, Entity};
use std::fmt;

#[derive(Debug)]
pub(super) enum InventoryAction {
    Examine,
    Take,
    Drop,
    Wield,
    Unwield,
}

impl fmt::Display for InventoryAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Examine => write!(f, "Examine"),
            Self::Take => write!(f, "Take"),
            Self::Drop => write!(f, "Drop"),
            Self::Wield => write!(f, "Wield"),
            Self::Unwield => write!(f, "Unwield"),
        }
    }
}

#[derive(Component, Debug)]
pub(super) struct InventoryButton(pub(super) Entity, pub(super) InventoryAction);
