use crate::gameplay::Amount;
use bevy::prelude::{Component, Entity};
use units::{Mass, Volume};

#[derive(Clone, Debug, Component)]
pub(crate) struct Containable {
    pub(crate) volume: Volume,
    pub(crate) mass: Mass,
}

#[derive(Component)]
pub(crate) struct ContainerLimits {
    pub(crate) max_volume: Volume,
    pub(crate) max_mass: Mass,
    pub(crate) max_amount: Option<Amount>,
}

#[derive(Debug, Component)]
pub(crate) struct BodyContainers {
    pub(crate) hands: Entity,
    pub(crate) clothing: Entity,
}

impl BodyContainers {
    pub(crate) fn default_hands_container_limits() -> ContainerLimits {
        ContainerLimits {
            max_volume: Volume::from("100 L"),
            max_mass: Mass::from("50 kg"),
            max_amount: Some(Amount::SINGLE),
        }
    }

    pub(crate) fn default_clothing_container_limits() -> ContainerLimits {
        ContainerLimits {
            max_volume: Volume::from("100 L"),
            max_mass: Mass::from("50 kg"),
            max_amount: None,
        }
    }
}
