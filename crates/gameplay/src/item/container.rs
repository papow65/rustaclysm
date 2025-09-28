use crate::item::phrases::{CanBearButNeeded, CanHoldButNeeded, HasButNeeded};
use crate::{Amount, Containable, InPocket, ItemHierarchy, LogMessageWriter, Subject};
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
        message_writer: &mut LogMessageWriter,
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
                message_writer.send(HasButNeeded {
                    subject: container_subject.clone(),
                    available: free_volume,
                    added: added.volume,
                });
            }

            if max_amount_by_mass == Amount::ZERO {
                message_writer.send(CanBearButNeeded {
                    subject: container_subject.clone(),
                    available: free_mass,
                    added: added.mass,
                });
            }

            if max_amount_by_amount == Amount::ZERO {
                message_writer.send(CanHoldButNeeded {
                    subject: container_subject,
                    available: max_amount_by_amount.0,
                    added: added_amount.0,
                });
            }

            Err(())
        }
    }
}
