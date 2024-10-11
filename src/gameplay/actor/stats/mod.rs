//! Stats for characters

mod health;
mod speed;
mod stamina;

pub(crate) use self::health::Health;
pub(crate) use self::speed::{BaseSpeed, WalkingMode};
pub(crate) use self::stamina::{Stamina, StaminaCost, StaminaImpact};

use bevy::prelude::Component;

/// For some animals
#[derive(Debug, Component)]
pub(crate) struct Aquatic;
