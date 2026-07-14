use bevy::prelude::{Component, TextColor};
use cdda_json_files::{CddaItemName, ItemName};
use gameplay_location::Pos;
use hud::BAD_TEXT_COLOR;
use std::sync::Arc;
use text::Fragment;

#[must_use]
#[derive(Clone, Debug, Component)]
#[component(immutable)]
pub struct ObjectName {
    name: ItemName,
    color: TextColor,
}

impl ObjectName {
    pub fn single(&self, pos: Pos) -> Fragment {
        Fragment::colorized(&*self.name.single, self.color).positioned(pos)
    }

    pub fn amount(&self, amount: u32, pos: Pos) -> Fragment {
        Fragment::colorized(&**self.name.amount(amount), self.color).positioned(pos)
    }

    pub const fn new(name: ItemName, color: TextColor) -> Self {
        Self { name, color }
    }

    pub fn from_str(text: &str, color: TextColor) -> Self {
        Self {
            name: ItemName::from(CddaItemName::Simple(Arc::from(text))),
            color,
        }
    }

    pub fn corpse() -> Self {
        Self::from_str("corpse", BAD_TEXT_COLOR)
    }

    pub fn missing() -> Self {
        Self::from_str("(missing name)", BAD_TEXT_COLOR)
    }
}
