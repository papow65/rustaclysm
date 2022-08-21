mod action;
mod faction;
mod player;
mod pos;

use crate::prelude::{Partial, Visible};
use bevy::prelude::{AlphaMode, Assets, Color, Component, Handle, StandardMaterial};

pub(crate) use {action::*, faction::*, player::*, pos::*};

#[derive(Component, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Label(pub(crate) String);

impl Label {
    pub(crate) fn new<S>(label: S) -> Self
    where
        S: Into<String>,
    {
        Self(label.into())
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.0.fmt(formatter)
    }
}

#[derive(Component)]
pub(crate) struct Floor;

#[derive(Component)]
pub(crate) struct Wall;

#[derive(Component)]
pub(crate) struct StairsUp;

#[derive(Component)]
pub(crate) struct StairsDown;

#[derive(Component)]
pub(crate) struct Window;

#[derive(Component)]
pub(crate) struct Rack;

#[derive(Component)]
pub(crate) struct Table;

#[derive(Component)]
pub(crate) struct Chair;

#[derive(Component)]
pub(crate) struct WindowPane;

#[derive(Component)]
pub(crate) struct Obstacle;

#[derive(Component)]
pub(crate) struct Hurdle(pub(crate) f32);

#[derive(Component)]
pub(crate) struct Opaque;

#[derive(Component)]
pub(crate) struct Container(pub(crate) u8);

#[derive(Component)]
pub(crate) struct Containable(pub(crate) u8);

#[derive(Component)]
pub(crate) struct Health {
    curr: i8,
    max: i8,
}

#[derive(Component, Debug, PartialEq, Eq)]
pub(crate) struct Item {
    pub(crate) amount: u32,
}

impl Health {
    pub(crate) fn new(max: i8) -> Self {
        assert!(0 < max);
        Self { curr: max, max }
    }

    const fn relative_damage(&self) -> Partial {
        let damage = self.max - self.curr;
        Partial::from_u8((255_i16 * damage as i16 / self.max as i16) as u8)
    }

    pub(crate) fn apply(&mut self, damage: &Damage) -> bool {
        self.curr -= damage.amount.min(self.curr).max(self.curr - self.max);
        0 < self.curr
    }
}

impl std::fmt::Display for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{curr}", curr = self.curr)
    }
}

#[derive(Component)]
pub(crate) struct Integrity {
    pub(crate) curr: i32,
    pub(crate) max: i32,
}

impl Integrity {
    pub(crate) const fn new(max: i32) -> Self {
        Self { curr: max, max }
    }

    // TODO de-duplicate code with Health::apply
    pub(crate) fn apply(&mut self, damage: &Damage) -> bool {
        self.curr -= i32::from(damage.amount)
            .min(self.curr)
            .max(self.max - self.curr);
        0 < self.curr
    }
}

impl std::fmt::Display for Integrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{curr}", curr = self.curr)
    }
}

#[derive(Component)]
pub(crate) struct Damage {
    pub(crate) attacker: Label,
    pub(crate) amount: i8, // TODO damage type
}

#[derive(Component)]
pub(crate) struct Corpse;

#[derive(Component)]
pub(crate) struct CameraBase;

#[derive(Component)]
pub(crate) struct ExamineCursor;

#[derive(Component, PartialEq, Eq, Debug)]
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
            _ => panic!("material(...) called when never seen"),
        }
    }

    fn remembered(color: Color) -> Color {
        Color::rgba(
            color.r() / 2.0,
            color.g() / 2.0,
            color.b() / 1.5,
            0.5f32.mul_add(color.a(), 0.5),
        )
    }
}

#[derive(Component)]
pub(crate) struct Message(pub(crate) String, pub(crate) Color); // shown to the player

impl Message {
    pub(crate) fn new<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Self(s.into(), Color::WHITE)
    }

    pub(crate) fn warn<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Self(s.into(), Color::rgb(1.0, 1.0, 0.4))
    }

    pub(crate) fn error<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Self(s.into(), Color::rgb(1.0, 0.4, 0.4))
    }
}

#[derive(Component)]
pub(crate) struct LogDisplay;

#[derive(Component)]
pub(crate) struct StatusDisplay;

#[derive(Component)]
pub(crate) struct ManualDisplay;
