use crate::prelude::{Containable, Mass, Message, TextLabel, Volume};

pub(crate) struct Container {
    pub(crate) max_volume: Volume,
    pub(crate) max_mass: Mass,
    pub(crate) max_amount: Option<usize>,
}

impl Container {
    pub(crate) fn check_add<'a, I>(
        &self,
        label: &TextLabel,
        current_items: I,
        added: &Containable,
        added_label: &TextLabel,
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
            messages.push(Message::warn(format!(
                "{label} has only {free_volume} space left, but {} needed to pick up {added_label}",
                added.volume
            )));
        }

        let free_mass = self.max_mass - current_mass;
        if free_mass < added.mass {
            messages.push(Message::warn(format!(
                "{label} can bear only {free_mass} more, but {} needed to pick up {added_label}",
                added.mass
            )));
        }

        if let Some(max_amount) = self.max_amount {
            let free_amount = max_amount - curent_amount;
            let added_amout = 1;
            if free_amount < added_amout {
                messages.push(Message::warn(format!(
                    "{label} can hold {}, but {added_amout} needed to pick up {added_label}",
                    match free_amount {
                        0 => String::from("no more items"),
                        1 => String::from("only one more item"),
                        _ => format!("only {free_amount} more items"),
                    }
                )));
            }
        }

        if messages.is_empty() {
            Ok(())
        } else {
            Err(messages)
        }
    }
}
