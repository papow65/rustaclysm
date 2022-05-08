mod action;
mod faction;
mod player;
mod pos;

use bevy::prelude::{AlphaMode, Assets, Color, Component, Handle, StandardMaterial};

pub use super::unit::Partial;

pub use action::{Action, Instruction};
pub use faction::{Faction, Intelligence};
pub use player::{Player, PlayerActionState};
pub use pos::{Path, Pos, PosYChanged, Zone, ZoneChanged, ZoneLevel};

#[derive(Component, Clone, Debug)]
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

#[derive(Component)]
pub struct Floor;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Stairs;

#[derive(Component)]
pub struct Window;

#[derive(Component)]
pub struct Rack;

#[derive(Component)]
pub struct Table;

#[derive(Component)]
pub struct Chair;

#[derive(Component)]
pub struct WindowPane;

#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct Hurdle(pub f32);

#[derive(Component)]
pub struct Opaque;

#[derive(Component)]
pub struct Container(pub u8);

#[derive(Component)]
pub struct Containable(pub u8);

#[derive(Component)]
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
        write!(f, "{curr}", curr = self.curr)
    }
}

#[derive(Component)]
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
pub struct Damage {
    pub attacker: Label,
    pub amount: i8, // TODO damage type
}

#[derive(Component)]
pub struct Corpse;

#[derive(Component)]
pub struct CameraBase;

#[derive(Component)]
pub struct ExamineCursor;

#[derive(Component, Clone, Copy, PartialEq)]
pub enum PlayerVisible {
    Seen,
    Hidden,
    Reevaluate,
}

impl PlayerVisible {
    pub fn adjust(&self, color: Color) -> Color {
        match self {
            PlayerVisible::Seen => color,
            PlayerVisible::Hidden => Color::rgba(
                color.r() / 4.0,
                color.g() / 4.0,
                color.b() / 3.0,
                0.5f32.mul_add(color.a(), 0.5),
            ),
            _ => panic!(),
        }
    }
}

#[derive(Component, Clone)]
pub struct Appearance {
    seen: Handle<StandardMaterial>,
    out_of_sight: Handle<StandardMaterial>,
}

impl Appearance {
    pub fn new<T>(materials: &mut Assets<StandardMaterial>, material: T) -> Self
    where
        T: Into<StandardMaterial>,
    {
        let mut material = material.into();
        material.alpha_mode = AlphaMode::Blend;
        let out_of_sight = materials.add(StandardMaterial {
            base_color_texture: material.base_color_texture.clone(),
            base_color: PlayerVisible::Hidden.adjust(material.base_color),
            alpha_mode: AlphaMode::Blend,
            ..StandardMaterial::default()
        });
        Self {
            seen: materials.add(material),
            out_of_sight,
        }
    }

    pub fn material(&self, player_visible: PlayerVisible) -> Handle<StandardMaterial> {
        match player_visible {
            PlayerVisible::Seen => self.seen.clone(),
            PlayerVisible::Hidden => self.out_of_sight.clone(),
            _ => panic!(),
        }
    }
}

#[derive(Component)]
pub struct Message(pub String); // shown to the player

impl Message {
    pub fn new<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Self(s.into())
    }
}

#[derive(Component)]
pub struct LogDisplay;

#[derive(Component)]
pub struct StatusDisplay;

#[derive(Component)]
pub struct ManualDisplay;
