use crate::Health;
use bevy::prelude::{Component, TextColor};
use gameplay_location::Pos;
use hud::{FILTHY_COLOR, HARD_TEXT_COLOR, WARN_TEXT_COLOR};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Intelligence {
    Dumb,
    Smart,
}

#[derive(Debug, Component)]
#[component(immutable)]
pub struct LastEnemy(pub Pos);

pub trait BaseFaction: Eq {
    fn is_aggressive(&self, health: &Health) -> bool;

    fn dislikes(&self, other: &Self) -> bool;

    fn can_fear(&self) -> bool;

    fn wanders(&self) -> bool;

    fn intelligence(&self) -> Intelligence;

    fn color(&self) -> TextColor;
}

#[derive(Clone, Debug, PartialEq, Eq, Component)]
#[component(immutable)]
pub enum Faction {
    Human,
    Zombie,
    Animal,
}

impl BaseFaction for Faction {
    fn is_aggressive(&self, health: &Health) -> bool {
        match self {
            Self::Human => health.value().relative() < 0.5,
            _ => true,
        }
    }

    fn dislikes(&self, other: &Self) -> bool {
        self != other
    }

    fn can_fear(&self) -> bool {
        self != &Self::Zombie
    }

    fn wanders(&self) -> bool {
        self != &Self::Human
    }

    fn intelligence(&self) -> Intelligence {
        match self {
            Self::Zombie => Intelligence::Dumb,
            _ => Intelligence::Smart,
        }
    }

    fn color(&self) -> TextColor {
        match self {
            Self::Human => HARD_TEXT_COLOR,
            Self::Zombie => FILTHY_COLOR,
            Self::Animal => WARN_TEXT_COLOR,
        }
    }
}
