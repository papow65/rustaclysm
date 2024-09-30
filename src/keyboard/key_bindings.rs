use crate::keyboard::{key_binding::KeyBinding, CtrlState, HeldState, Key};
use crate::manual::ManualSection;
use bevy::prelude::{Commands, ComputedStates, IntoSystem, StateScoped, States, World};
use std::{cell::OnceCell, iter::once};

#[derive(Clone)]
struct KeyBindingsStorage<S: States, C: CtrlState, H: HeldState> {
    bindings: Vec<KeyBinding<C, H>>,
    manual: ManualSection,
    state: S,
}

impl<S: States, C: CtrlState, H: HeldState> KeyBindingsStorage<S, C, H> {
    fn spawn(self, commands: &mut Commands) {
        let scoped = StateScoped(self.state);
        commands.spawn((self.manual, scoped.clone()));
        commands.spawn_batch(
            self.bindings
                .into_iter()
                .map(move |binding| (binding, scoped.clone())),
        );
    }
}

pub(crate) struct KeyBindingsBuilder<'w, S: States, C: CtrlState, H: HeldState> {
    storage: KeyBindingsStorage<S, C, H>,
    world: &'w mut World,
}

impl<'w, S: States, C: CtrlState, H: HeldState> KeyBindingsBuilder<'w, S, C, H> {
    pub(crate) fn add<K: Into<Key>, M, T: IntoSystem<Key, (), M> + 'static>(
        &mut self,
        key: K,
        system: T,
    ) {
        self.add_multi(once(key), system);
    }

    pub(crate) fn add_multi<
        K: Into<Key>,
        M,
        T: IntoSystem<Key, (), M> + 'static,
        V: IntoIterator<Item = K>,
    >(
        &mut self,
        keys: V,
        system: T,
    ) {
        self.storage.bindings.push(KeyBinding::new(
            keys.into_iter().map(Into::into).collect(),
            self.world.register_system(system),
        ));
    }
}

#[derive(Default)]
pub(crate) struct KeyBindings<S: States, C: CtrlState, H: HeldState> {
    once: OnceCell<KeyBindingsStorage<S, C, H>>,
}

impl<S: States, C: CtrlState, H: HeldState> KeyBindings<S, C, H> {
    pub(crate) fn spawn<F>(&self, world: &mut World, state: S, init: F, manual: ManualSection)
    where
        F: FnOnce(&mut KeyBindingsBuilder<S, C, H>),
    {
        let storage = self.once.get_or_init(|| {
            let mut builder = KeyBindingsBuilder {
                storage: KeyBindingsStorage {
                    bindings: Vec::new(),
                    manual,
                    state,
                },
                world,
            };
            init(&mut builder);
            builder.storage
        });
        storage.clone().spawn(&mut world.commands());
        world.flush(); // apply commands
    }
}

#[derive(Clone, Default, PartialEq, Eq, Hash, Debug)]
pub(crate) struct GlobalState;

impl ComputedStates for GlobalState {
    type SourceStates = Self;

    fn compute(_: Self) -> Option<Self> {
        Some(Self)
    }
}

impl<C: CtrlState, H: HeldState> KeyBindings<GlobalState, C, H> {
    pub(crate) fn spawn_global<F>(world: &mut World, init: F, manual: ManualSection)
    where
        F: FnOnce(&mut KeyBindingsBuilder<GlobalState, C, H>),
    {
        Self::default().spawn(world, GlobalState, init, manual);
    }
}
