use crate::prelude::{Containable, Fragment, Mass, Message, Phrase, Volume};
use bevy::prelude::{Component, Entity};

#[derive(Component)]
pub(crate) struct Container {
    pub(crate) max_volume: Volume,
    pub(crate) max_mass: Mass,
    pub(crate) max_amount: Option<usize>,
}

impl Container {
    pub(crate) fn check_add<'a, I>(
        &self,
        container_name: Fragment,
        current_items: I,
        added: &Containable,
        added_name: Vec<Fragment>,
    ) -> Result<(), Vec<Message>>
    where
        I: Iterator<Item = &'a Containable>,
    {
        let mut messages = Vec::new();

        let (current_volume, current_mass, curent_amount) = current_items
            .fold((Volume::ZERO, Mass::ZERO, 0), |acc, item| {
                (acc.0 + item.volume, acc.1 + item.mass, acc.2 + 1)
            });

        let free_volume = self.max_volume - current_volume;
        if free_volume < added.volume {
            let added_volume = added.volume;
            if free_volume == Volume::ZERO {
                String::from("no space left")
            } else {
                format!("only {free_volume} available")
            };
            messages.push(Message::warn(
                Phrase::from_fragment(container_name.clone())
                    .add(format!(
                        "has {free_volume}, but {added_volume} needed to pick up"
                    ))
                    .extend(added_name.clone()),
            ));
        }

        let free_mass = self.max_mass - current_mass;
        if free_mass < added.mass {
            let added_mass = added.mass;
            let free_mass = if free_mass == Mass::ZERO {
                String::from("no more weight")
            } else {
                format!("only {free_mass} more")
            };
            messages.push(Message::warn(
                Phrase::from_fragment(container_name.clone())
                    .add(format!(
                        "can bear {free_mass}, but {added_mass} needed to pick up",
                    ))
                    .extend(added_name.clone()),
            ));
        }

        if let Some(max_amount) = self.max_amount {
            let free_amount = max_amount.saturating_sub(curent_amount);
            let added_amout = 1;
            if free_amount < added_amout {
                let free_amount = match free_amount {
                    0 => String::from("no more items"),
                    1 => String::from("only one more item"),
                    _ => format!("only {free_amount} more items"),
                };
                messages.push(Message::warn(
                    Phrase::from_fragment(container_name)
                        .add(format!(
                            "can hold {free_amount}, but {added_amout} needed to pick up",
                        ))
                        .extend(added_name),
                ));
            }
        }

        if messages.is_empty() {
            Ok(())
        } else {
            Err(messages)
        }
    }
}

#[derive(Component)]
pub(crate) struct BodyContainers {
    pub(crate) hands: Entity,
    pub(crate) clothing: Entity,
}

impl BodyContainers {
    pub(crate) fn default_hands_container() -> Container {
        Container {
            max_volume: Volume::from(String::from("100 L")),
            max_mass: Mass::from(String::from("50 kg")),
            max_amount: Some(1),
        }
    }

    pub(crate) fn default_clothing_container() -> Container {
        Container {
            max_volume: Volume::from(String::from("100 L")),
            max_mass: Mass::from(String::from("50 kg")),
            max_amount: None,
        }
    }
}
