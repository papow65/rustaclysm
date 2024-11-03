use crate::gameplay::{
    Amount, Containable, Filthy, Fragment, ItemIntegrity, ObjectDefinition, ObjectName, Pos,
    Positioning,
};
use bevy::ecs::query::QueryData;
use bevy::prelude::{Entity, Parent};

#[derive(QueryData)]
#[query_data(derive(Debug))]
pub(crate) struct Item {
    pub(crate) entity: Entity,
    pub(crate) definition: &'static ObjectDefinition,
    pub(crate) name: &'static ObjectName,
    pub(crate) pos: Option<&'static Pos>,
    pub(crate) amount: &'static Amount,
    pub(crate) filthy: Option<&'static Filthy>,
    pub(crate) integrity: &'static ItemIntegrity,
    pub(crate) containable: &'static Containable,
    pub(crate) parent: &'static Parent,
}

impl<'a> ItemItem<'a> {
    pub(crate) fn fragments(&self) -> impl Iterator<Item = Fragment> + use<'_, 'a> {
        [
            self.amount.fragment(),
            self.filthy.map(|_| Filthy::fragment()),
            self.integrity.fragment(),
            Some(self.name.amount(self.amount.0, Pos::ORIGIN)),
        ]
        .into_iter()
        .flatten()
        .map(|mut fragment| {
            fragment.positioning = if let Some(&pos) = self.pos {
                Positioning::Pos(pos)
            } else {
                Positioning::None
            };
            fragment
        })
    }
}
