use crate::prelude::*;
use bevy::prelude::{Color, Component};

#[derive(Clone, Component, Debug)]
pub(crate) struct ObjectName {
    name: ItemName,
    color: Color,
}

impl ObjectName {
    #[must_use]
    pub(crate) fn single(&self) -> Fragment {
        Fragment::colorized(self.name.single.clone(), self.color)
    }

    #[must_use]
    pub(crate) fn as_item(
        &self,
        amount: Option<&Amount>,
        filthy: Option<&Filthy>,
    ) -> Vec<Fragment> {
        let amount = match amount {
            Some(Amount(n)) => *n,
            _ => 1,
        };
        let mut result = Vec::new();
        if 1 < amount {
            result.push(Fragment::new(format!("{amount}")));
        }
        if filthy.is_some() {
            result.push(Fragment::colorized("filthy", FILTHY_COLOR));
        }
        result.push(Fragment::colorized(
            if amount == 1 {
                self.name.single.clone()
            } else {
                self.name.plural.clone()
            },
            self.color,
        ));
        result
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
