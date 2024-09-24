use std::sync::Arc;

use crate::common::BAD_TEXT_COLOR;
use crate::gameplay::{Fragment, Pos};
use bevy::prelude::{Color, Component};
use cdda_json_files::{CddaItemName, ItemName};

#[derive(Clone, Component, Debug)]
pub(crate) struct ObjectName {
    name: ItemName,
    color: Color,
}

impl ObjectName {
    #[must_use]
    pub(crate) fn single(&self, pos: Pos) -> Fragment {
        Fragment::positioned(&*self.name.single, self.color, pos)
    }

    #[must_use]
    pub(crate) fn amount(&self, amount: u32, pos: Pos) -> Fragment {
        Fragment::positioned(&**self.name.amount(amount), self.color, pos)
    }

    #[must_use]
    pub(crate) const fn new(name: ItemName, color: Color) -> Self {
        Self { name, color }
    }

    #[must_use]
    pub(crate) fn from_str(text: &str, color: Color) -> Self {
        Self {
            name: ItemName::from(CddaItemName::Simple(Arc::from(text))),
            color,
        }
    }

    #[must_use]
    pub(crate) fn corpse() -> Self {
        Self::from_str("corpse", BAD_TEXT_COLOR)
    }

    #[must_use]
    pub(crate) fn missing() -> Self {
        Self::from_str("(missing name)", BAD_TEXT_COLOR)
    }
}
