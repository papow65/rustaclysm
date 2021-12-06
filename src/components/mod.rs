mod action;
mod factions;
mod player;
mod pos;

use bevy::prelude::{Assets, Color, Handle, StandardMaterial};

pub use super::units::Partial;

pub use action::{Action, Instruction};
pub use factions::{Faction, Intelligence};
pub use player::Player;
pub use pos::{Path, Pos, SIZE};

#[derive(Clone, Debug)]
pub struct Label(pub String);

impl Label {
    pub fn new<S>(label: S) -> Self
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

pub struct Floor;
pub struct Wall;
pub struct Stairs;
pub struct Window;
pub struct Rack;
pub struct Table;
pub struct Chair;

pub struct WindowPane;
pub struct StairsDown;

pub struct Obstacle;
pub struct Hurdle(pub f32);

pub struct Opaque;

pub struct Container(pub u8);
pub struct Containable(pub u8);

pub struct Health {
    curr: i8,
    max: i8,
}

impl Health {
    pub fn new(max: i8) -> Self {
        assert!(0 < max);
        Self { curr: max, max }
    }

    const fn relative_damage(&self) -> Partial {
        let damage = self.max - self.curr;
        Partial::from_u8((255_i16 * damage as i16 / self.max as i16) as u8)
    }

    pub fn apply(&mut self, damage: &Damage) -> bool {
        self.curr -= damage.amount.min(self.curr).max(self.curr - self.max);
        0 < self.curr
    }
}

impl std::fmt::Display for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.curr)
    }
}

pub struct Integrity {
    pub curr: i32,
    pub max: i32,
}

impl Integrity {
    pub const fn new(max: i32) -> Self {
        Self { curr: max, max }
    }

    // TODO de-duplicate code with Health::apply
    pub fn apply(&mut self, damage: &Damage) -> bool {
        self.curr -= (damage.amount as i32)
            .min(self.curr)
            .max(self.max - self.curr);
        0 < self.curr
    }
}

impl std::fmt::Display for Integrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.curr)
    }
}

pub struct Damage {
    pub attacker: Label,
    pub amount: i8, // TODO damage type
}

pub struct Corpse;

pub struct InheritVisibility;

#[derive(Clone, Copy, PartialEq)]
pub enum Visibility {
    Seen,
    Hidden,
    Reevaluate,
}

impl Visibility {
    pub fn adjust(&self, color: Color) -> Color {
        match self {
            Visibility::Seen => color,
            Visibility::Hidden => Color::rgba(
                color.r() / 4.0,
                color.g() / 4.0,
                color.b() / 3.0,
                0.5 + 0.5 * color.a(),
            ),
            _ => panic!(),
        }
    }
}

#[derive(Clone)]
pub struct Appearance {
    seen: Handle<StandardMaterial>,
    out_of_sight: Handle<StandardMaterial>,
}

impl Appearance {
    pub fn new<T>(materials: &mut Assets<StandardMaterial>, material: T) -> Self
    where
        T: Into<StandardMaterial>,
    {
        let material = material.into();
        let out_of_sight = materials.add(StandardMaterial {
            base_color_texture: material.base_color_texture.clone(),
            base_color: Visibility::Hidden.adjust(material.base_color),
            ..StandardMaterial::default()
        });
        Self {
            seen: materials.add(material),
            out_of_sight,
        }
    }

    pub fn material(&self, visibility: Visibility) -> Handle<StandardMaterial> {
        match visibility {
            Visibility::Seen => self.seen.clone(),
            Visibility::Hidden => self.out_of_sight.clone(),
            _ => panic!(),
        }
    }
}

pub struct PosYChanged;

pub struct Status; // debugging

pub struct Message(pub String); // shown to the player

impl Message {
    pub const fn new(s: String) -> (Self,) {
        (Self(s),)
    }
}

pub struct LogDisplay;
pub struct StatusDisplay;
pub struct ManualDisplay;
