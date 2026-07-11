use crate::messages::{CanBearButNeeded, CanHoldButNeeded, HasButNeeded};
use crate::{Amount, Containable, InPocket, ItemHierarchy};
use gameplay_log::LogMessageWriter;
use text::Subject;
use units::{Mass, Volume};

pub struct AdditionFailure {
    volume: Option<HasButNeeded>,
    mass: Option<CanBearButNeeded>,
    amount: Option<CanHoldButNeeded>,
}

impl AdditionFailure {
    pub fn write(self, message_writer: &mut LogMessageWriter) {
        if let Some(volume) = self.volume {
            message_writer.send(volume);
        }
        if let Some(mass) = self.mass {
            message_writer.send(mass);
        }
        if let Some(amount) = self.amount {
            message_writer.send(amount);
        }
    }
}

impl
    TryFrom<(
        Option<HasButNeeded>,
        Option<CanBearButNeeded>,
        Option<CanHoldButNeeded>,
    )> for AdditionFailure
{
    type Error = ();

    fn try_from(
        (volume, mass, amount): (
            Option<HasButNeeded>,
            Option<CanBearButNeeded>,
            Option<CanHoldButNeeded>,
        ),
    ) -> Result<Self, Self::Error> {
        if volume.is_none() && mass.is_none() && amount.is_none() {
            Err(())
        } else {
            Ok(Self {
                volume,
                mass,
                amount,
            })
        }
    }
}

pub struct Container<'a> {
    pub in_pocket: InPocket,
    hierarchy: &'a ItemHierarchy<'a, 'a>,
}

impl<'a> Container<'a> {
    #[must_use]
    pub const fn new(in_pocket: InPocket, hierarchy: &'a ItemHierarchy) -> Self {
        Self {
            in_pocket,
            hierarchy,
        }
    }

    /// # Errors
    /// When there are not enough availiable volume, mass, or slots
    pub fn check_add(
        &self,
        container_subject: Subject,
        added: &Containable,
        added_amount: Amount,
    ) -> Result<Amount, AdditionFailure> {
        // TODO check that the added item is not already part of the current contents or the full ancestry of the container

        let limits = self.hierarchy.container(self.in_pocket);

        let (current_volume, current_mass, current_amount) = self
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
        let volmue_error = (max_amount_by_volume == Amount::ZERO).then_some(HasButNeeded {
            subject: container_subject.clone(),
            available: free_volume,
            added: added.volume,
        });

        let free_mass = limits.max_mass - current_mass;
        let max_amount_by_mass = if Mass::ZERO < added.mass {
            Amount(free_mass / added.mass)
        } else {
            added_amount
        };
        let mass_error = (max_amount_by_mass == Amount::ZERO).then_some(CanBearButNeeded {
            subject: container_subject.clone(),
            available: free_mass,
            added: added.mass,
        });

        let max_amount_by_amount = if let Some(max_amount) = limits.max_amount {
            &max_amount - &current_amount
        } else {
            added_amount
        };
        let count_error = (max_amount_by_amount == Amount::ZERO).then_some(CanHoldButNeeded {
            subject: container_subject,
            available: max_amount_by_amount.0,
            added: added_amount.0,
        });

        match AdditionFailure::try_from((volmue_error, mass_error, count_error)) {
            Ok(err) => Err(err),
            Err(()) => Ok(max_amount_by_volume
                .min(max_amount_by_mass)
                .min(max_amount_by_amount)),
        }
    }
}
