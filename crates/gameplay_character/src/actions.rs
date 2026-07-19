use crate::{Actor, ActorItem};
use bevy::prelude::{Entity, Query};
use gameplay_crafting::RecipeSituation;
use gameplay_item::{Item, ItemItem};
use gameplay_location::{CardinalDirection, HorizontalDirection, Nbor};

/// An action that an actor can perform
pub trait Action: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug)]
pub struct ActionIn<A: Action> {
    pub actor_entity: Entity,
    pub action: A,
}

impl<A: Action> ActionIn<A> {
    pub const fn new(actor_entity: Entity, action: A) -> Self {
        Self {
            actor_entity,
            action,
        }
    }

    pub fn actor<'a>(&self, actors: &'a Query<Actor>) -> ActorItem<'a, 'a> {
        actors.get(self.actor_entity).expect("Actor entity")
    }
}

#[derive(Clone, Debug)]
pub struct Stay;

impl Action for Stay {}

#[derive(Clone, Debug)]
pub struct Sleep;

impl Action for Sleep {}

#[derive(Clone, Debug)]
pub struct Step {
    pub to: Nbor,
}

impl Action for Step {}

#[derive(Clone, Debug)]
pub struct Attack {
    pub target: Nbor,
}

impl Action for Attack {}

#[derive(Clone, Debug)]
pub struct Smash {
    pub target: Nbor,
}

impl Action for Smash {}

#[derive(Clone, Debug)]
pub struct Pulp {
    pub target: HorizontalDirection,
}

impl Action for Pulp {}

#[derive(Clone, Debug)]
pub struct Peek {
    pub target: CardinalDirection,
}

impl Action for Peek {}

#[derive(Clone, Debug)]
pub struct Close {
    pub target: HorizontalDirection,
}

impl Action for Close {}

pub trait ItemAction: Action {
    fn item_entity(&self) -> Entity;

    fn item<'a>(&self, items: &'a Query<Item>) -> ItemItem<'a, 'a> {
        items.get(self.item_entity()).expect("Item entity")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wield {
    pub item_entity: Entity,
}

impl Action for Wield {}

impl ItemAction for Wield {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unwield {
    pub item_entity: Entity,
}

impl Action for Unwield {}

impl ItemAction for Unwield {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pickup {
    pub item_entity: Entity,
}

impl Action for Pickup {}

impl ItemAction for Pickup {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MoveItem {
    pub item_entity: Entity,
    pub to: Nbor,
}

impl Action for MoveItem {}

impl ItemAction for MoveItem {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Debug)]
pub struct StartCraft {
    pub recipe_situation: RecipeSituation,
    pub target: HorizontalDirection,
}

impl Action for StartCraft {}

#[derive(Clone, Debug)]
pub struct ContinueCraft {
    pub item_entity: Entity,
}

impl Action for ContinueCraft {}

impl ItemAction for ContinueCraft {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

/// Redundantly named to avoid confusion
#[derive(Clone, Debug, PartialEq)]
pub struct ExamineItem {
    pub item_entity: Entity,
}

impl Action for ExamineItem {}

impl ItemAction for ExamineItem {
    fn item_entity(&self) -> Entity {
        self.item_entity
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChangePace {
    Next,
    Previous,
}

impl Action for ChangePace {}
