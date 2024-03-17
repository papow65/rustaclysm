use crate::prelude::{CddaItemName, Fragment, ItemName, Pos, BAD_TEXT_COLOR};
use bevy::prelude::{Color, Component};

#[derive(Clone, Component, Debug)]
pub(crate) struct ObjectName {
    name: ItemName,
    color: Color,
}

impl ObjectName {
    #[must_use]
    pub(crate) fn single(&self, pos: Pos) -> Fragment {
        Fragment::positioned(self.name.single.clone(), self.color, pos)
    }

    #[must_use]
    pub(crate) fn plural(&self, pos: Pos) -> Fragment {
        Fragment::positioned(self.name.plural.clone(), self.color, pos)
    }

    #[must_use]
    pub(crate) const fn new(name: ItemName, color: Color) -> Self {
        Self { name, color }
    }

    #[must_use]
    pub(crate) fn from_str(text: &str, color: Color) -> Self {
        Self {
            name: ItemName::from(CddaItemName::Simple(String::from(text))),
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
