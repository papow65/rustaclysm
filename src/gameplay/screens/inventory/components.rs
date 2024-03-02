use bevy::prelude::{Component, Entity};
use std::fmt;

#[derive(Debug)]
pub(crate) enum InventoryAction {
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
pub(crate) struct ActionButton(pub(crate) Entity, pub(crate) InventoryAction);

#[derive(Component, Default)]
pub(crate) struct ScrollingList {
    pub(crate) position: f32,
}
