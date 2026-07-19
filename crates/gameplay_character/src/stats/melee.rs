use bevy::prelude::Component;
use cdda_json_files::CommonItemInfo;
use fastrand::u16 as rand_u16;
use gameplay_common::Shared;

#[derive(Debug, Component)]
#[component(immutable)]
pub struct Melee {
    /// Can be 0
    pub(crate) dices: u16,

    /// Can be 0
    pub(crate) sides: u16,
}

impl Melee {
    pub(crate) fn damage(&self, melee_weapon: Option<&Shared<CommonItemInfo>>) -> u16 {
        (1..=self.dices)
            .map(|_| {
                rand_u16(
                    1..=self.sides
                        + melee_weapon.map_or(0, |common_info| common_info.melee_damage()),
                )
            })
            .sum()
    }
}
