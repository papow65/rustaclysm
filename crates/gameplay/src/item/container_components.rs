use crate::{Amount, InPocket};
use bevy::prelude::Component;
use units::{Mass, Volume};

#[derive(Clone, Debug, Component)]
#[component(immutable)]
pub(crate) struct Containable {
    pub(crate) volume: Volume,
    pub(crate) mass: Mass,
}

#[derive(Component)]
#[component(immutable)]
pub(crate) struct ContainerLimits {
    pub(crate) max_volume: Volume,
    pub(crate) max_mass: Mass,
    pub(crate) max_amount: Option<Amount>,
}

#[derive(Debug, Component)]
#[component(immutable)]
pub(crate) struct BodyContainers {
    pub(crate) hands: InPocket,
    pub(crate) clothing: InPocket,
}

impl BodyContainers {
    pub(crate) const fn all(&self) -> [InPocket; 2] {
        [self.hands, self.clothing]
    }

    pub(crate) fn default_hands_container_limits() -> ContainerLimits {
        ContainerLimits {
            max_volume: Volume::try_from("100 L").expect("Well formatted"),
            max_mass: Mass::try_from("50 kg").expect("Well formatted"),
            max_amount: Some(Amount::SINGLE),
        }
    }

    pub(crate) fn default_clothing_container_limits() -> ContainerLimits {
        ContainerLimits {
            max_volume: Volume::try_from("100 L").expect("Well formatted"),
            max_mass: Mass::try_from("50 kg").expect("Well formatted"),
            max_amount: None,
        }
    }
}
