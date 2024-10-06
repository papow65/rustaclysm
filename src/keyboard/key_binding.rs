use crate::keyboard::{CtrlState, HeldState, Key};
use bevy::ecs::system::SystemId;
use bevy::prelude::{Component, Entity, In};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub(crate) enum KeyBindingSystem {
    Simple(SystemId<(), ()>),
    Key(SystemId<In<Key>, ()>),
    Entity(SystemId<In<Entity>, ()>),
}

impl From<SystemId<(), ()>> for KeyBindingSystem {
    fn from(system: SystemId<(), ()>) -> Self {
        Self::Simple(system)
    }
}

impl From<SystemId<In<Key>, ()>> for KeyBindingSystem {
    fn from(system: SystemId<In<Key>, ()>) -> Self {
        Self::Key(system)
    }
}

impl From<SystemId<In<Entity>, ()>> for KeyBindingSystem {
    fn from(system: SystemId<In<Entity>, ()>) -> Self {
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
