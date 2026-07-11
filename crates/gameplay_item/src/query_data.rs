use crate::{Amount, Containable, Filthy, InPocket, ItemIntegrity, Phase, Pockets};
use bevy::ecs::query::QueryData;
use bevy::prelude::{Children, Entity, ops::atan2};
use cdda_json_files::{CommonItemInfo, InfoId};
use either::Either;
use gameplay_common::ObjectName;
use gameplay_common::Shared;
use gameplay_location::Pos;
use gameplay_relations::ObjectOn;
use hud::text_color_expect_half;
use std::f32::consts::FRAC_1_PI;
use text::{Fragment, Positioning};

#[derive(QueryData)]
#[query_data(derive(Debug))]
pub struct Item {
    pub entity: Entity,
    pub name: &'static ObjectName,
    pub pos: Option<&'static Pos>,
    pub amount: &'static Amount,
    pub filthy: Option<&'static Filthy>,
    pub integrity: &'static ItemIntegrity,
    pub phase: &'static Phase,
    pub containable: &'static Containable,
    pub on_tile: Option<&'static ObjectOn>,
    pub in_pocket: Option<&'static InPocket>,
    pub pockets: Option<&'static Pockets>,
    pub models: Option<&'static Children>,
    pub common_info: &'static Shared<CommonItemInfo>,
}

impl<'w, 's> ItemItem<'w, 's> {
    #[must_use]
    pub fn parentage(&self) -> Either<&ObjectOn, &InPocket> {
        match (self.on_tile, self.in_pocket) {
            (None, None) => panic!(
                "No tile or pocket for item {:?} at {:?}",
                self.name, self.pos
            ),
            (None, Some(in_pocket)) => Either::Right(in_pocket),
            (Some(on_tile), None) => Either::Left(on_tile),
            (Some(on_tile), Some(in_pocket)) => {
                panic!(
                    "Both a tile ({on_tile:?}) and a pocket ({in_pocket:?}) for item {:?} at {:?}",
                    self.name, self.pos
                )
            }
        }
    }

    pub fn fragments(&self) -> impl Iterator<Item = Fragment> + use<'_, 'w, 's> {
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
