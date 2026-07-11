use crate::{Amount, InPocket};
use bevy::prelude::Component;
use units::{Mass, Volume};

#[derive(Clone, Debug, Component)]
#[component(immutable)]
pub struct Containable {
    pub volume: Volume,
    pub mass: Mass,
}

#[derive(Component)]
#[component(immutable)]
pub struct ContainerLimits {
    pub max_volume: Volume,
    pub max_mass: Mass,
    pub max_amount: Option<Amount>,
}

#[derive(Debug, Component)]
#[component(immutable)]
pub struct BodyContainers {
    pub hands: InPocket,
    pub clothing: InPocket,
}

impl BodyContainers {
    #[must_use]
    pub const fn all(&self) -> [InPocket; 2] {
        [self.hands, self.clothing]
    }

    #[must_use]
    pub fn default_hands_container_limits() -> ContainerLimits {
        ContainerLimits {
            max_volume: Volume::try_from("100 L").expect("Well formatted"),
            max_mass: Mass::try_from("50 kg").expect("Well formatted"),
            max_amount: Some(Amount::SINGLE),
        }
    }

    #[must_use]
    pub fn default_clothing_container_limits() -> ContainerLimits {
        ContainerLimits {
            max_volume: Volume::try_from("100 L").expect("Well formatted"),
            max_mass: Mass::try_from("50 kg").expect("Well formatted"),
            max_amount: None,
        }
    }
}
