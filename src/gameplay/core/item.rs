use crate::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};

#[derive(QueryData)]
#[query_data(derive(Debug))]
pub(crate) struct Item {
    pub(crate) entity: Entity,
    pub(crate) definition: &'static ObjectDefinition,
    pub(crate) name: &'static ObjectName,
    pub(crate) pos: Option<&'static Pos>,
    pub(crate) amount: &'static Amount,
    pub(crate) filthy: Option<&'static Filthy>,
    pub(crate) containable: &'static Containable,
    pub(crate) parent: &'static Parent,
}

impl ItemItem<'_> {
    #[must_use]
    pub(crate) fn fragments(&self) -> Vec<Fragment> {
        let mut result = Vec::new();
        if &Amount::SINGLE < self.amount {
            result.push(Fragment::new(format!("{}", self.amount.0)));
        }
        if self.filthy.is_some() {
            result.push(Fragment::colorized("filthy", FILTHY_COLOR));
        }
        result.push(if self.amount == &Amount::SINGLE {
            self.name.single(Pos::ORIGIN)
        } else {
            self.name.plural(Pos::ORIGIN)
        });

        for fragment in &mut result {
            fragment.positioning = if let Some(&pos) = self.pos {
                Positioning::Pos(pos)
            } else {
                Positioning::None
            };
        }

        result
    }
}
