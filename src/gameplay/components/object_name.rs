use std::sync::Arc;

use crate::gameplay::{Fragment, Pos};
use bevy::prelude::{Component, TextColor};
use cdda_json_files::{CddaItemName, ItemName};
use hud::BAD_TEXT_COLOR;

#[must_use]
#[derive(Clone, Component, Debug)]
pub(crate) struct ObjectName {
    name: ItemName,
    color: TextColor,
}

impl ObjectName {
    pub(crate) fn single(&self, pos: Pos) -> Fragment {
        Fragment::colorized(&*self.name.single, self.color).positioned(pos)
    }

    pub(crate) fn amount(&self, amount: u32, pos: Pos) -> Fragment {
        Fragment::colorized(&**self.name.amount(amount), self.color).positioned(pos)
    }

    pub(crate) const fn new(name: ItemName, color: TextColor) -> Self {
        Self { name, color }
    }

    pub(crate) fn from_str(text: &str, color: TextColor) -> Self {
        Self {
            name: ItemName::from(CddaItemName::Simple(Arc::from(text))),
            color,
        }
    }

    pub(crate) fn corpse() -> Self {
        Self::from_str("corpse", BAD_TEXT_COLOR)
    }

    pub(crate) fn missing() -> Self {
        Self::from_str("(missing name)", BAD_TEXT_COLOR)
    }
}
