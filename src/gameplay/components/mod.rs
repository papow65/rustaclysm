mod faction;
mod message;
mod player;
mod pos;
mod stats;

use crate::prelude::*;
use bevy::{
    pbr::StandardMaterial,
    prelude::{AlphaMode, Assets, Color, Component, Handle},
};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};

pub(crate) use {faction::*, message::*, player::*, pos::*, stats::*};

#[derive(Component)]
pub(crate) struct Filthy;

#[derive(Component)]
pub(crate) struct ManualRoot;

#[derive(Component)]
pub(crate) struct Accessible {
    pub(crate) water: bool,
    pub(crate) move_cost: MoveCost,
}

#[derive(Component)]
pub(crate) struct StairsUp;

#[derive(Component)]
pub(crate) struct StairsDown;

/** Not accessible for any movement */
#[derive(Component)]
pub(crate) struct Obstacle;

#[derive(Component)]
pub(crate) struct Openable;

#[derive(Component)]
pub(crate) struct Closeable;

#[derive(Component)]
pub(crate) struct Hurdle(pub(crate) MoveCostMod);

/** Blocks vision (horizontally) */
#[derive(Component)]
pub(crate) struct Opaque;

/** Blocks vision to and from the level below */
#[derive(Component)]
pub(crate) struct OpaqueFloor;

#[derive(Component)]
pub(crate) struct Containable {
    pub(crate) volume: Volume,
    pub(crate) mass: Mass,
}

#[derive(Component, Debug)]
pub(crate) struct Amount(pub(crate) u32);

/** Marker to open or close something, like a door */
#[derive(Component)]
pub(crate) struct Toggle;

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

#[derive(Component)]
pub(crate) struct Integrity(pub(crate) Limited);

impl Integrity {
    pub(crate) fn apply(&mut self, damage: &Damage) -> bool {
        self.0.saturating_subtract(damage.amount);
        self.0.is_nonzero()
    }
}

#[derive(Component)]
pub(crate) struct Damage {
    pub(crate) attacker: Fragment, // for logging
    pub(crate) amount: i16,        // TODO damage types
}

#[derive(Component)]
pub(crate) struct Corpse;

#[derive(Component)]
pub(crate) struct CameraBase;

#[derive(Component)]
pub(crate) struct ExamineCursor;

#[derive(Component)]
pub(crate) struct Hands(pub(crate) Container);

impl Default for Hands {
    fn default() -> Self {
        Self(Container {
            max_volume: Volume::from(String::from("100 L")),
            max_mass: Mass::from(String::from("50 kg")),
            max_amount: Some(1),
        })
    }
}

#[derive(Component)]
pub(crate) struct Clothing(pub(crate) Container);

impl Default for Clothing {
    fn default() -> Self {
        Self(Container {
            max_volume: Volume::from(String::from("100 L")),
            max_mass: Mass::from(String::from("50 kg")),
            max_amount: None,
        })
    }
}

#[derive(Component)]
pub(crate) struct Melee {
    pub(crate) dices: u16,
    pub(crate) sides: u16,
}

impl Melee {
    pub(crate) fn damage(&self) -> i16 {
        assert!(0 < self.dices, "{}", self.dices);
        assert!(0 < self.sides, "{}", self.sides);
        let mut rng = thread_rng();
        let between = Uniform::from(1..=self.sides);
        (1..=self.dices)
            .map(|_| between.sample(&mut rng))
            .sum::<u16>() as i16
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

    pub(crate) fn shown(&self, can_move: bool) -> bool {
        // Things that can move, like NPCs, are hidden when out of sight.
        self == &Self::Currently || (self == &Self::Previously && !can_move)
    }
}

/** Indication for a zone level that it only show its overmap tile
A zone level without this indicates that it is expanded into tiles */
#[derive(Component)]
pub(crate) struct Collapsed;

#[derive(Component, Clone)]
pub(crate) struct Appearance {
    seen: Handle<StandardMaterial>,
    remembered: Handle<StandardMaterial>,
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
            seen: materials.add(material),
            remembered,
        }
    }

    pub(crate) fn material(&self, last_seen: &LastSeen) -> Handle<StandardMaterial> {
        match last_seen {
            LastSeen::Currently => self.seen.clone(),
            LastSeen::Previously => self.remembered.clone(),
            LastSeen::Never => panic!("material(...) called when never seen"),
        }
    }

    fn remembered(color: Color) -> Color {
        Color::rgba(
            color.r() / 1.4,
            color.g() / 1.4,
            color.b() / 1.2,
            0.5f32.mul_add(color.a(), 0.5),
        )
    }
}

#[derive(Component)]
pub(crate) struct StatusDisplay;

#[derive(Component)]
pub(crate) struct LogDisplay;

#[derive(Component)]
pub(crate) struct ManualDisplay;
