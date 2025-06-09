use crate::{Amount, Containable, InPocket, ItemHierarchy, MessageWriter, Subject};
use units::{Mass, Volume};

pub(crate) struct Container<'a> {
    pub(crate) in_pocket: InPocket,
    hierarchy: &'a ItemHierarchy<'a, 'a>,
}

impl<'a> Container<'a> {
    pub(crate) const fn new(in_pocket: InPocket, hierarchy: &'a ItemHierarchy) -> Self {
        Self {
            in_pocket,
            hierarchy,
        }
    }

    pub(crate) fn check_add(
        &self,
        message_writer: &mut MessageWriter,
        container_subject: Subject,
        added: &Containable,
        added_amount: Amount,
    ) -> Result<Amount, ()> {
        // TODO check that the added item is not already part of the current contents or the full ancestry of the container

        let limits = self.hierarchy.container(self.in_pocket);

        let (current_volume, current_mass, curent_amount) = self
            .hierarchy
            .items_in_pocket(self.in_pocket)
            .fold((Volume::ZERO, Mass::ZERO, Amount::ZERO), |acc, item| {
                (
                    acc.0 + item.containable.volume,
                    acc.1 + item.containable.mass,
                    &acc.2 + item.amount,
                )
            });

        let free_volume = limits.max_volume - current_volume;
        let max_amount_by_volume = if Volume::ZERO < added.volume {
            Amount(free_volume / added.volume)
        } else {
            added_amount
        };

        let free_mass = limits.max_mass - current_mass;
        let max_amount_by_mass = if Mass::ZERO < added.mass {
            Amount(free_mass / added.mass)
        } else {
            added_amount
        };

        let max_amount_by_amount = if let Some(max_amount) = limits.max_amount {
            &max_amount - &curent_amount
        } else {
            added_amount
        };

        let allowed_amount = max_amount_by_volume
            .min(max_amount_by_mass)
            .min(max_amount_by_amount);

        if Amount::ZERO < allowed_amount {
            Ok(allowed_amount)
        } else {
            if max_amount_by_volume == Amount::ZERO {
                let added_volume = added.volume;
                if free_volume == Volume::ZERO {
                    String::from("no space left")
                } else {
                    format!("only {free_volume} available")
                };
                message_writer
                    .subject(container_subject.clone())
                    .simple(format!("has {free_volume}, but {added_volume} needed").as_str())
                    .send_warn();
            }

            if max_amount_by_mass == Amount::ZERO {
                let added_mass = added.mass;
                let free_mass = if free_mass == Mass::ZERO {
                    String::from("no more weight")
                } else {
                    format!("only {free_mass} more")
                };
                message_writer
                    .subject(container_subject.clone())
                    .simple(format!("can bear {free_mass}, but {added_mass} needed").as_str())
                    .send_warn();
            }

            if max_amount_by_amount == Amount::ZERO {
                let free_amount = match max_amount_by_amount.0 {
                    0 => String::from("no more items"),
                    1 => String::from("only one more item"),
                    _ => format!("only {} more items", max_amount_by_amount.0),
                };
                message_writer
                    .subject(container_subject)
                    .simple(
                        format!("can hold {free_amount}, but {} needed", added_amount.0).as_str(),
                    )
                    .send_warn();
            }

            Err(())
        }
    }
}
