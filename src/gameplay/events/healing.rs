use crate::gameplay::events::ActorChange;

#[derive(Clone, Debug)]
pub(crate) struct Healing {
    pub(crate) amount: u16,
}

impl ActorChange for Healing {}
