use crate::gameplay::Fragment;
use crate::hud::text_color_expect_full;
use bevy::prelude::Component;

#[derive(Debug, Component)]
pub(crate) struct ItemIntegrity {
    damage: i64,
}

impl ItemIntegrity {
    // Based on itype.h:1311
    const BROKEN_DAMAGE: i64 = 4000;
    const REINFORCED_DAMAGE: i64 = -1000;

    pub(crate) const fn broken(&self) -> bool {
        Self::BROKEN_DAMAGE <= self.damage
    }

    pub(crate) fn fragment(&self) -> Option<Fragment> {
        //trace!("{self:?}");
        Some(Fragment::colorized(
            match self.damage {
                damage if damage <= Self::REINFORCED_DAMAGE => "!!",
                damage if Self::BROKEN_DAMAGE <= damage => "broken",
                damage if Self::BROKEN_DAMAGE * 4 / 5 <= damage => "..",
                damage if Self::BROKEN_DAMAGE * 3 / 5 <= damage => "./",
                damage if Self::BROKEN_DAMAGE * 2 / 5 <= damage => ".|",
                damage if Self::BROKEN_DAMAGE / 5 <= damage => "/|",
                _ => {
                    return None;
                }
            },
            text_color_expect_full(
                1.0 - self.damage.clamp(0, Self::BROKEN_DAMAGE) as f32 / Self::BROKEN_DAMAGE as f32,
            ),
        ))
    }
}

impl From<Option<i64>> for ItemIntegrity {
    fn from(source: Option<i64>) -> Self {
        Self {
            damage: source.unwrap_or(0),
        }
    }
}
