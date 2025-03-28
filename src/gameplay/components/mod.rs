mod object_name;
mod pos;
mod shared;
mod vehicle;

pub(crate) use self::object_name::ObjectName;
pub(crate) use self::pos::{Level, Overzone, Pos, SubzoneLevel, Zone, ZoneLevel};
pub(crate) use self::shared::Shared;
pub(crate) use self::vehicle::{Vehicle, VehiclePart};

use crate::gameplay::{BaseSpeed, Damage, Evolution, Limited, Player, Visible};
use bevy::prelude::{AlphaMode, Assets, Color, Component, MeshMaterial3d, Srgba, StandardMaterial};
use cdda_json_files::{CommonItemInfo, MoveCost, MoveCostIncrease, Recipe};
use std::sync::Arc;
use units::{Duration, Timestamp};

/// Terrain that can be accessed, like a floor
#[derive(Component)]
#[component(immutable)]
pub(crate) struct Accessible {
    pub(crate) water: bool,
    pub(crate) move_cost: MoveCost,
}

#[derive(Component)]
#[component(immutable)]
pub(crate) struct StairsUp;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct StairsDown;

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

/// Mutable component
#[derive(Debug, PartialEq, Component)]
pub(crate) struct StandardIntegrity(pub(crate) Limited);

impl StandardIntegrity {
    pub(crate) fn lower(&mut self, damage: &Damage) -> Evolution {
        self.0.lower(damage.amount)
    }
    // TODO raising (not with Healing)
}

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

#[derive(Component)]
#[component(immutable)]
pub(crate) struct PlayerWielded;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct CameraBase;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct ExamineCursor;

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

/// Mutable component
#[derive(Debug, Component)]
pub(crate) struct Craft {
    pub(crate) recipe: Arc<Recipe>,
    pub(crate) work_needed: Duration,
    pub(crate) work_done: Duration,
}

impl Craft {
    pub(crate) const fn new(recipe: Arc<Recipe>, work_needed: Duration) -> Self {
        Self {
            recipe,
            work_needed,
            work_done: Duration::ZERO,
        }
    }

    pub(crate) fn work(&mut self, duration: Duration) {
        self.work_done += duration;
    }

    pub(crate) const fn finished(&self) -> bool {
        self.work_needed.milliseconds() <= self.work_done.milliseconds()
    }

    pub(crate) fn percent_progress(&self) -> f32 {
        100.0 * self.work_done.milliseconds() as f32 / self.work_needed.milliseconds() as f32
    }

    pub(crate) fn time_left(&self) -> Duration {
        self.work_needed - self.work_done
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

/// Mutable component
#[derive(Clone, PartialEq, Eq, Debug, Component)]
pub(crate) enum LastSeen {
    Currently,
    Previously, // TODO consider adding a timestamp
    Never,
}

impl LastSeen {
    pub(crate) fn update(&mut self, visible: &Visible) {
        if visible == &Visible::Seen {
            *self = Self::Currently;
        } else if self == &Self::Currently {
            *self = Self::Previously;
        }
    }

    pub(crate) fn shown(&self, player: Option<&Player>, speed: Option<&BaseSpeed>) -> bool {
        // Things that can move, like NPCs, are hidden when out of sight.
        self == &Self::Currently
            || (self == &Self::Previously && speed.is_none())
            || player.is_some()
    }
}

#[derive(Clone, Debug, Component)]
#[component(immutable)]
pub(crate) struct Appearance {
    seen: MeshMaterial3d<StandardMaterial>,
    remembered: MeshMaterial3d<StandardMaterial>,
}

impl Appearance {
    pub(crate) fn new<T>(materials: &mut Assets<StandardMaterial>, material: T) -> Self
    where
        T: Into<StandardMaterial>,
    {
        let mut material = material.into();
        material.alpha_mode = AlphaMode::Blend;
        let remembered = materials.add(StandardMaterial {
            base_color_texture: material.base_color_texture.clone(),
            base_color: Self::remembered(material.base_color),
            alpha_mode: AlphaMode::Blend,
            ..StandardMaterial::default()
        });
        Self {
            seen: materials.add(material).into(),
            remembered: remembered.into(),
        }
    }

    pub(crate) fn fixed_material(&self) -> MeshMaterial3d<StandardMaterial> {
        self.material(&LastSeen::Currently)
    }

    pub(crate) fn material(&self, last_seen: &LastSeen) -> MeshMaterial3d<StandardMaterial> {
        match last_seen {
            LastSeen::Currently => self.seen.clone(),
            LastSeen::Previously => self.remembered.clone(),
            LastSeen::Never => panic!("material(...) called when never seen"),
        }
    }

    fn remembered(color: Color) -> Color {
        let srgba = Srgba::from(color);
        Color::srgba(
            srgba.red * 0.6,
            srgba.green * 0.6,
            srgba.blue,
            0.5_f32.mul_add(srgba.alpha, 0.5),
        )
    }
}

#[derive(Component)]
#[component(immutable)]
pub(crate) struct MissingAsset;
