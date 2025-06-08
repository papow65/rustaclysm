use crate::{Actor, ActorItem, Item, ItemItem, RecipeSituation};
use bevy::prelude::{Entity, Query};
use gameplay_location::{CardinalDirection, HorizontalDirection, Nbor};

/// An action that an actor can perform
pub(crate) trait Action: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug)]
pub(crate) struct ActionIn<A: Action> {
    pub(crate) actor_entity: Entity,
    pub(crate) action: A,
}

impl<A: Action> ActionIn<A> {
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
pub(crate) struct Stay;

impl Action for Stay {}

#[derive(Clone, Debug)]
pub(crate) struct Sleep;

impl Action for Sleep {}

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
pub(crate) struct Peek {
    pub(crate) target: CardinalDirection,
}

impl Action for Peek {}

#[derive(Clone, Debug)]
pub(crate) struct Close {
    pub(crate) target: HorizontalDirection,
}

impl Action for Close {}

pub(crate) trait ItemAction: Action {
    fn item_entity(&self) -> Entity;

    fn item<'a>(&self, items: &'a Query<Item>) -> ItemItem<'a> {
        items.get(self.item_entity()).expect("Item entity")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Wield {
    pub(crate) item_entity: Entity,
}

impl Action for Wield {}

impl ItemAction for Wield {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Unwield {
    pub(crate) item_entity: Entity,
}

impl Action for Unwield {}

impl ItemAction for Unwield {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Pickup {
    pub(crate) item_entity: Entity,
}

impl Action for Pickup {}

impl ItemAction for Pickup {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MoveItem {
    pub(crate) item_entity: Entity,
    pub(crate) to: Nbor,
}

impl Action for MoveItem {}

impl ItemAction for MoveItem {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StartCraft {
    pub(crate) recipe_situation: RecipeSituation,
    pub(crate) target: HorizontalDirection,
}

impl Action for StartCraft {}

#[derive(Clone, Debug)]
pub(crate) struct ContinueCraft {
    pub(crate) item_entity: Entity,
}

impl Action for ContinueCraft {}

impl ItemAction for ContinueCraft {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

/// Redundantly named to avoid confusion
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExamineItem {
    pub(crate) item_entity: Entity,
}

impl Action for ExamineItem {}

impl ItemAction for ExamineItem {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ChangePace {
    Next,
    Previous,
}

impl Action for ChangePace {}
