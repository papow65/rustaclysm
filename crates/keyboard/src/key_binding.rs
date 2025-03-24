use crate::{CtrlState, HeldState, Key};
use bevy::ecs::system::SystemId;
use bevy::prelude::{Component, Entity, In};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub enum KeyBindingSystem {
    Simple(SystemId<(), ()>),
    Entity(SystemId<In<Entity>, ()>),
}

impl From<SystemId<(), ()>> for KeyBindingSystem {
    fn from(system: SystemId<(), ()>) -> Self {
        Self::Simple(system)
    }
}

impl From<SystemId<In<Entity>, ()>> for KeyBindingSystem {
    fn from(system: SystemId<In<Entity>, ()>) -> Self {
        Self::Entity(system)
    }
}

#[derive(Clone, Debug, Component)]
pub struct KeyBinding<C: CtrlState, H: HeldState> {
    key: Key,
    system: KeyBindingSystem,
    _phantom: PhantomData<(C, H)>,
}

impl<C: CtrlState, H: HeldState> KeyBinding<C, H> {
    #[must_use]
    pub const fn new(key: Key, system: KeyBindingSystem) -> Self {
        Self {
            key,
            system,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn matching_system(&self, key: Key) -> Option<&KeyBindingSystem> {
        (self.key == key).then_some(&self.system)
    }
}
