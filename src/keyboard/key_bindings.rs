use crate::keyboard::{key_binding::KeyBinding, CtrlState, HeldState, Key};
use crate::manual::ManualSection;
use bevy::prelude::{ComputedStates, IntoSystem, StateScoped, States, World};
use std::cell::OnceCell;

struct KeyBindingsStorage<S: States, C: CtrlState, H: HeldState> {
    bindings: Vec<KeyBinding<C, H>>,
    manual: ManualSection,
    state: S,
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
        self.add_multi([key], system);
    }

    pub(crate) fn add_multi<
        K: Into<Key>,
        M,
        T: IntoSystem<Key, (), M> + 'static,
        const N: usize,
    >(
        &mut self,
        keys: [K; N],
        system: T,
    ) {
        self.storage.bindings.push(KeyBinding::new(
            keys.map(Into::into),
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
        let builder = self.once.get_or_init(|| {
            let mut bindings = KeyBindingsBuilder {
                storage: KeyBindingsStorage {
                    bindings: Vec::new(),
                    manual,
                    state,
                },
                world,
            };
            init(&mut bindings);
            bindings.storage
        });

        world.spawn_batch(
            builder
                .bindings
                .iter()
                .cloned()
                .map(|binding| (binding, StateScoped(builder.state.clone()))),
        );

        world.spawn((builder.manual.clone(), StateScoped(builder.state.clone())));
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
