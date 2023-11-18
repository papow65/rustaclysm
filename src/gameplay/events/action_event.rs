use crate::{
    gameplay::HorizontalDirection,
    prelude::{Actor, ActorItem, Item, ItemItem, Nbor, StayDuration},
};
use bevy::prelude::{Entity, Event, Query};

/** An action that an actor can perform */
pub(crate) trait Action: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug, Event)]
pub(crate) struct ActionEvent<A: Action> {
    pub(crate) actor_entity: Entity,
    pub(crate) action: A,
}

impl<A: Action> ActionEvent<A> {
    pub(crate) const fn new(actor_entity: Entity, action: A) -> Self {
        Self {
            actor_entity,
            action,
        }
    }

    pub(crate) fn actor<'a>(&self, actors: &'a Query<Actor>) -> ActorItem<'a> {
        actors.get(self.actor_entity).expect("Actor entity")
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Stay {
    pub(crate) duration: StayDuration,
}

impl Action for Stay {}

#[derive(Clone, Debug)]
pub(crate) struct Step {
    pub(crate) to: Nbor,
}

impl Action for Step {}

#[derive(Clone, Debug)]
pub(crate) struct Attack {
    pub(crate) target: Nbor,
}

impl Action for Attack {}

#[derive(Clone, Debug)]
pub(crate) struct Smash {
    pub(crate) target: Nbor,
}

impl Action for Smash {}

#[derive(Clone, Debug)]
pub(crate) struct Pulp {
    pub(crate) target: HorizontalDirection,
}

impl Action for Pulp {}

#[derive(Clone, Debug)]
pub(crate) struct Close {
    pub(crate) target: HorizontalDirection,
}

impl Action for Close {}

pub(crate) trait ItemChange: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug, Event)]
pub(crate) struct ItemAction<C: ItemChange> {
    pub(crate) item_entity: Entity,
    pub(crate) change: C,
}

impl<C: ItemChange> ItemAction<C> {
    pub(crate) const fn new(item_entity: Entity, change: C) -> Self {
        Self {
            item_entity,
            change,
        }
    }

    pub(crate) fn item<'a>(&self, items: &'a Query<Item>) -> ItemItem<'a> {
        items.get(self.item_entity).expect("Actor entity")
    }
}

impl<C: ItemChange> Action for ItemAction<C> {}

#[derive(Clone, Debug)]
pub(crate) struct Wield;

impl ItemChange for Wield {}

#[derive(Clone, Debug)]
pub(crate) struct Unwield;

impl ItemChange for Unwield {}

#[derive(Clone, Debug)]
pub(crate) struct Pickup;

impl ItemChange for Pickup {}

/** Redundantly named to avoid confusion */
#[derive(Clone, Debug)]
pub(crate) struct MoveItem {
    pub(crate) to: Nbor,
}

impl ItemChange for MoveItem {}

/** Redundantly named to avoid confusion */
#[derive(Clone, Debug)]
pub(crate) struct ExamineItem;

impl ItemChange for ExamineItem {}

#[derive(Clone, Debug)]
pub(crate) struct ChangePace;

impl Action for ChangePace {}
