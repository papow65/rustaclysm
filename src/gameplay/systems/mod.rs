mod input;
mod shutdown;
mod spawn;
mod startup;
mod update;

pub(crate) use self::{input::*, shutdown::*, spawn::*, startup::*, update::*};

use crate::gameplay::*;
use bevy::prelude::*;

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn count_entities(
    all: Query<()>,
    zone_levels: Query<(), With<ZoneLevel>>,
    subzone_levels: Query<(), With<SubzoneLevel>>,
    pos: Query<(), With<Pos>>,
) {
    let total = all.iter().len();
    let subzone_levels = subzone_levels.iter().len();
    let zone_levels = zone_levels.iter().len();
    let pos = pos.iter().len();
    let other = total - subzone_levels - zone_levels - pos;

    println!("{subzone_levels} zone levels, {zone_levels} subzone levels, {pos} positions, and {other} other entities");
}
