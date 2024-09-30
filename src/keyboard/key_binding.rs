use crate::keyboard::{CtrlState, HeldState, Key};
use bevy::ecs::system::SystemId;
use bevy::prelude::Component;
use std::marker::PhantomData;

#[derive(Clone, Debug, Component)]
pub(super) struct KeyBinding<C: CtrlState, H: HeldState> {
    keys: Vec<Key>,
    system: SystemId<Key, ()>,
    _phantom: PhantomData<(C, H)>,
}

impl<C: CtrlState, H: HeldState> KeyBinding<C, H> {
    pub(super) const fn new(keys: Vec<Key>, system: SystemId<Key, ()>) -> Self {
        Self {
            keys,
            system,
            _phantom: PhantomData,
        }
    }

    pub(super) fn matching_system(&self, key: Key) -> Option<SystemId<Key, ()>> {
        self.keys
            .iter()
            .copied()
            .any(|k| k == key)
            .then_some(self.system)
    }
}
