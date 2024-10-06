mod container_limits;
mod object_name;
mod pos;
mod vehicle;

pub(crate) use container_limits::{BodyContainers, ContainerLimits};
pub(crate) use object_name::ObjectName;
pub(crate) use pos::{Level, Overzone, Pos, SubzoneLevel, Zone, ZoneLevel};
pub(crate) use vehicle::{Vehicle, VehiclePart};

use crate::gameplay::*;
use bevy::prelude::{AlphaMode, Assets, Color, Component, MeshMaterial3d, Srgba, StandardMaterial};
use cdda_json_files::{ItemInfo, MoveCost, MoveCostIncrease, ObjectId};
use std::ops::{Add, Sub};
use units::{Duration, Mass, Timestamp, Volume};

#[derive(PartialEq, Debug, Component)]
pub(crate) struct Filthy;

/// Terrain that can be accessed, like a floor
#[derive(Component)]
pub(crate) struct Accessible {
    pub(crate) water: bool,
    pub(crate) move_cost: MoveCost,
}

#[derive(Component)]
pub(crate) struct StairsUp;

#[derive(Component)]
pub(crate) struct StairsDown;

/// Not accessible for any movement
#[derive(Component)]
pub(crate) struct Obstacle;

#[derive(Component)]
pub(crate) struct Openable;

#[derive(Component)]
pub(crate) struct Closeable;

#[derive(Component)]
pub(crate) struct Hurdle(pub(crate) MoveCostIncrease);

/// Blocks vision (horizontally)
#[derive(Component)]
pub(crate) struct Opaque;

/// Blocks vision to and from the level below
#[derive(Component)]
pub(crate) struct OpaqueFloor;

#[derive(Clone, Debug, Component)]
pub(crate) struct Containable {
    pub(crate) volume: Volume,
    pub(crate) mass: Mass,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Amount(pub(crate) u32);

impl Amount {
    pub(crate) const ZERO: Self = Self(0);
    pub(crate) const SINGLE: Self = Self(1);
}

impl Add<Self> for &Amount {
    type Output = Amount;
    fn add(self, other: Self) -> Self::Output {
        Amount(self.0 + other.0)
    }
}

impl Sub<Self> for &Amount {
    type Output = Amount;
    fn sub(self, other: Self) -> Self::Output {
        Amount(self.0 - other.0)
    }
}

#[derive(Clone, Component, Debug, PartialEq)]
pub(crate) struct ObjectDefinition {
    pub(crate) category: ObjectCategory,
    pub(crate) id: ObjectId,
}

impl ObjectDefinition {
    pub(crate) fn alpha_mode(&self) -> AlphaMode {
        if self.category == ObjectCategory::Terrain && self.id.is_ground() {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        }
    }
}

#[derive(PartialEq, Component)]
pub(crate) struct Integrity(pub(crate) Limited);

impl Integrity {
    pub(crate) fn lower(&mut self, damage: &Damage) -> Evolution {
        self.0.lower(damage.amount)
    }
    // TODO raising (not with Healing)
}

#[derive(Debug, Component)]
pub(crate) struct Life;

#[derive(Component)]
pub(crate) struct Corpse;

#[derive(Component)]
pub(crate) struct CorpseRaise {
    pub(crate) at: Timestamp,
}

#[derive(Component)]
pub(crate) struct PlayerWielded;

#[derive(Component)]
pub(crate) struct CameraBase;

#[derive(Component)]
pub(crate) struct ExamineCursor;

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
pub(crate) struct Craft {
    pub(crate) object_id: ObjectId,
    pub(crate) work_needed: Duration,
    pub(crate) work_done: Duration,
}

impl Craft {
    pub(crate) const fn new(object_id: ObjectId, work_needed: Duration) -> Self {
        Self {
            object_id,
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
pub(crate) struct Melee {
    pub(crate) dices: u16,
    pub(crate) sides: u16,
}

impl Melee {
    pub(crate) fn damage(&self, melee_weapon: Option<&ItemInfo>) -> u16 {
        assert!(0 < self.dices, "{}", self.dices);
        assert!(0 < self.sides, "{}", self.sides);
        (1..=self.dices)
            .map(|_| fastrand::u16(1..=self.sides + melee_weapon.map_or(0, ItemInfo::melee_damage)))
            .sum()
    }
}

#[derive(Component, Clone, PartialEq, Eq, Debug)]
pub(crate) enum LastSeen {
    Currently,
    Previously, // TODO add timestamp
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

#[derive(Component, Clone)]
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
pub(crate) struct MissingAsset;
