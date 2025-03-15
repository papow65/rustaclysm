use crate::gameplay::{
    Amount, Containable, Filthy, Fragment, ItemIntegrity, ObjectDefinition, ObjectName, Pos,
    Positioning, Shared,
};
use crate::hud::text_color_expect_half;
use bevy::ecs::query::QueryData;
use bevy::prelude::{Entity, Parent, ops::atan2};
use cdda_json_files::{CommonItemInfo, InfoId};
use std::f32::consts::FRAC_1_PI;

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
    pub(crate) common_info: &'static Shared<CommonItemInfo>,
}

impl<'a> ItemItem<'a> {
    pub(crate) fn fragments(&self) -> impl Iterator<Item = Fragment> + use<'_, 'a> {
        let fragments = if self.common_info.id == InfoId::new("money") {
            let cents = self.amount.0 as f32;
            let dollars = format!("$ {:.2}", cents / 100.0);

            // $ 100 is treated as the expected amount
            let score = 2.0 * FRAC_1_PI * atan2(cents, 10000.0);
            let color = text_color_expect_half(score);

            [
                None,
                self.filthy.map(|_| Filthy::fragment()),
                self.integrity.fragment(),
                Some(Fragment::colorized(dollars, color)),
            ]
        } else if self.common_info.id == InfoId::new("battery") {
            [self.amount.fragment(), None, None, None]
        } else {
            [
                self.amount.fragment(),
                self.filthy.map(|_| Filthy::fragment()),
                self.integrity.fragment(),
                Some(self.name.amount(self.amount.0, Pos::ORIGIN)),
            ]
        };

        fragments.into_iter().flatten().map(|mut fragment| {
            fragment.positioning = if let Some(&pos) = self.pos {
                Positioning::Pos(pos)
            } else {
                Positioning::None
            };
            fragment
        })
    }
}
