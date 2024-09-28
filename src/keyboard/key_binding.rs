use crate::keyboard::{Ctrl, CtrlState, Held, HeldState, Key};
use bevy::ecs::system::SystemId;
use bevy::prelude::{Component, StateScoped, States};
use std::marker::PhantomData;

#[derive(Debug, Component)]
pub(crate) struct KeyBinding<C: CtrlState, H: HeldState> {
    keys: Vec<Key>, // TODO SmallVec
    system: SystemId<Key, ()>,
    _phantom: PhantomData<(C, H)>,
}

impl KeyBinding<(), ()> {
    pub(crate) const fn new(keys: Vec<Key>, system: SystemId<Key, ()>) -> Self {
        Self {
            keys,
            system,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn from<K: Into<Key>>(key: K, system: SystemId<Key, ()>) -> Self {
        Self::from_multi([key], system)
    }

    pub(crate) fn from_multi<K: Into<Key>, const N: usize>(
        keys: [K; N],
        system: SystemId<Key, ()>,
    ) -> Self {
        Self::new(keys.map(Into::into).into(), system)
    }
}

impl<C: CtrlState> KeyBinding<C, ()> {
    pub(crate) fn held(self) -> KeyBinding<C, Held> {
        KeyBinding {
            keys: self.keys,
            system: self.system,
            _phantom: PhantomData,
        }
    }
}

impl<H: HeldState> KeyBinding<(), H> {
    pub(crate) fn with_ctrl(self) -> KeyBinding<Ctrl, H> {
        KeyBinding {
            keys: self.keys,
            system: self.system,
            _phantom: PhantomData,
        }
    }
}

impl<C: CtrlState, H: HeldState> KeyBinding<C, H> {
    pub(super) fn matching_system(&self, key: Key) -> Option<SystemId<Key, ()>> {
        self.keys
            .iter()
            .copied()
            .any(|k| k == key)
            .then_some(self.system)
    }

    pub(crate) const fn scoped<S: States>(self, state: S) -> (Self, StateScoped<S>) {
        (self, StateScoped(state))
    }
}
