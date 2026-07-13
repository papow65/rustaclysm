mod last_seen_ext;

pub(crate) use self::last_seen_ext::LastSeenExt;

use bevy::prelude::Component;
use cdda_json_files::{CommonItemInfo, MoveCost, MoveCostIncrease};
use gameplay_common::Shared;
use units::{Duration, Timestamp};

/// Terrain that can be accessed, like a floor
#[derive(Component)]
#[component(immutable)]
pub(crate) struct Accessible {
    pub(crate) water: bool,
    pub(crate) move_cost: MoveCost,
}

/// Not accessible for any movement
#[derive(Component)]
#[component(immutable)]
pub(crate) struct Obstacle;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct Openable;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct Closeable;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct Hurdle(pub(crate) MoveCostIncrease);

/// Blocks vision (horizontally)
#[derive(Component)]
#[component(immutable)]
pub(crate) struct Opaque;

/// Blocks vision to and from the level below
#[derive(Component)]
#[component(immutable)]
pub(crate) struct OpaqueFloor;

#[derive(Debug, Component)]
#[component(immutable)]
pub(crate) struct Life;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct Corpse;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct CorpseRaise {
    pub(crate) at: Timestamp,
}

/// Mutable component
#[derive(Debug, Component)]
pub(crate) struct HealingDuration(Duration);

impl HealingDuration {
    pub(crate) const fn new() -> Self {
        Self(Duration::ZERO)
    }

    #[must_use]
    pub(crate) fn heal(&mut self, duration: Duration) -> u64 {
        let healing_rate = Duration::SECOND * 1000;

        self.0 += duration;
        self.0.extract_div(healing_rate)
    }
}

#[derive(Debug, Component)]
#[component(immutable)]
pub(crate) struct Melee {
    /// Can be 0
    pub(crate) dices: u16,

    /// Can be 0
    pub(crate) sides: u16,
}

impl Melee {
    pub(crate) fn damage(&self, melee_weapon: Option<&Shared<CommonItemInfo>>) -> u16 {
        (1..=self.dices)
            .map(|_| {
                fastrand::u16(
                    1..=self.sides
                        + melee_weapon.map_or(0, |common_info| common_info.melee_damage()),
                )
            })
            .sum()
    }
}

#[derive(Debug, Component)]
#[component(immutable)]
pub(crate) struct MissingAsset;

/// Used to indicate a root for all non-mobile objects with the same position
#[derive(Debug, Component)]
#[component(immutable)]
pub(crate) struct Tile;
