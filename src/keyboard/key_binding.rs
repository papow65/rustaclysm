use crate::keyboard::{CtrlState, HeldState, Key};
use bevy::ecs::system::SystemId;
use bevy::prelude::{Component, Entity};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub(crate) enum KeyBindingSystem {
    Simple(SystemId<(), ()>),
    Key(SystemId<Key, ()>),
    Entity(SystemId<Entity, ()>),
}

impl From<SystemId<(), ()>> for KeyBindingSystem {
    fn from(system: SystemId<(), ()>) -> Self {
        Self::Simple(system)
    }
}

impl From<SystemId<Key, ()>> for KeyBindingSystem {
    fn from(system: SystemId<Key, ()>) -> Self {
        Self::Key(system)
    }
}

impl From<SystemId<Entity, ()>> for KeyBindingSystem {
    fn from(system: SystemId<Entity, ()>) -> Self {
        Self::Entity(system)
    }
}

#[derive(Clone, Debug, Component)]
pub(crate) struct KeyBinding<C: CtrlState, H: HeldState> {
    keys: Vec<Key>,
    system: KeyBindingSystem,
    _phantom: PhantomData<(C, H)>,
}

impl<C: CtrlState, H: HeldState> KeyBinding<C, H> {
    pub(crate) const fn new(keys: Vec<Key>, system: KeyBindingSystem) -> Self {
        Self {
            keys,
            system,
            _phantom: PhantomData,
        }
    }

    pub(super) fn matching_system(&self, key: Key) -> Option<&KeyBindingSystem> {
        self.keys
            .iter()
            .copied()
            .any(|k| k == key)
            .then_some(&self.system)
    }
}
