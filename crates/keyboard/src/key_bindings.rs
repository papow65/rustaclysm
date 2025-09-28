use crate::key_binding::{KeyBinding, KeyBindingSystem};
use crate::{CtrlState, HeldState, Key};
use bevy::ecs::system::SystemId;
use bevy::prelude::{
    Commands, ComputedStates, DespawnOnExit, IntoSystem, States, SystemInput, World,
};
use std::cell::OnceCell;

#[derive(Clone, Debug)]
struct KeyBindingsStorage<S: States, C: CtrlState, H: HeldState> {
    bindings: Vec<KeyBinding<C, H>>,
    state: S,
}

impl<S: States, C: CtrlState, H: HeldState> KeyBindingsStorage<S, C, H> {
    fn spawn(self, commands: &mut Commands) {
        commands.spawn_batch(
            self.bindings
                .into_iter()
                .map(move |binding| (binding, DespawnOnExit(self.state.clone()))),
        );
    }
}

pub struct KeyBindingsBuilder<'w, S: States, C: CtrlState, H: HeldState> {
    storage: KeyBindingsStorage<S, C, H>,
    world: &'w mut World,
}

impl<S: States, C: CtrlState, H: HeldState> KeyBindingsBuilder<'_, S, C, H> {
    pub fn add<I: SystemInput + 'static, M>(
        &mut self,
        key: impl Into<Key>,
        system: impl IntoSystem<I, (), M> + 'static,
    ) where
        SystemId<I, ()>: Into<KeyBindingSystem>,
    {
        self.storage.bindings.push(KeyBinding::new(
            key.into(),
            self.world.register_system(system).into(),
        ));
    }
}

#[derive(Default)]
pub struct KeyBindings<S: States, C: CtrlState, H: HeldState> {
    once: OnceCell<KeyBindingsStorage<S, C, H>>,
}

impl<S: States, C: CtrlState, H: HeldState> KeyBindings<S, C, H> {
    pub fn spawn<F>(&self, world: &mut World, state: S, init: F)
    where
        F: FnOnce(&mut KeyBindingsBuilder<S, C, H>),
    {
        let storage = self.once.get_or_init(|| {
            let mut builder = KeyBindingsBuilder {
                storage: KeyBindingsStorage {
                    bindings: Vec::new(),
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
pub struct GlobalState;

impl ComputedStates for GlobalState {
    type SourceStates = Self;

    fn compute(_: Self) -> Option<Self> {
        Some(Self)
    }
}

impl<C: CtrlState, H: HeldState> KeyBindings<GlobalState, C, H> {
    pub fn spawn_global<F>(world: &mut World, init: F)
    where
        F: FnOnce(&mut KeyBindingsBuilder<GlobalState, C, H>),
    {
        Self::default().spawn(world, GlobalState, init);
    }
}
