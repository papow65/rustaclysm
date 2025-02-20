use crate::keyboard::key_binding::{KeyBinding, KeyBindingSystem};
use crate::keyboard::{Ctrl, CtrlState, Held, HeldState, InputChange, Key, KeyChange};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key as LogicalKey, KeyboardInput};
use bevy::prelude::{ButtonInput, Commands, Entity, EventReader, KeyCode, Query};

/// This resource contains all user keyboard input
///
/// The keys are updated every frame, but this rate may differ from the keyboard input from bevy.
/// So a key may not continuously be present in 'held', even when it is held down on the physical keyboard.
#[derive(Debug)]
pub(super) struct Keys {
    key_changes: Vec<KeyChange>,
    ctrl: Option<Ctrl>,
}

impl Keys {
    pub(super) fn new(
        keyboard_inputs: &mut EventReader<KeyboardInput>,
        key_states: &ButtonInput<KeyCode>,
    ) -> Self {
        let ctrl = (key_states.pressed(KeyCode::ControlLeft)
            || key_states.pressed(KeyCode::ControlRight))
        .then_some(Ctrl);
        let ctrl_change = key_states.just_pressed(KeyCode::ControlLeft)
            || key_states.just_pressed(KeyCode::ControlRight);

        Self {
            key_changes: keyboard_inputs
            .read()
            .filter_map(|keyboard_input| match keyboard_input {
                KeyboardInput { state: ButtonState::Released, .. } => { None}
                KeyboardInput { key_code, logical_key: LogicalKey::Character(key), .. } if !matches!(key_code, KeyCode::Numpad1 | KeyCode::Numpad2 | KeyCode::Numpad3 | KeyCode::Numpad4 | KeyCode::Numpad5 | KeyCode::Numpad6 | KeyCode::Numpad7 | KeyCode::Numpad8 | KeyCode::Numpad9)=> {
                    let mut chars = key.chars();
                    if let Some(char) = chars.next() {
                        if chars.next().is_some() {
                            eprintln!("Could not process keyboard input {keyboard_input:?}, because it's multiple characters.");
                            None
                        } else {
                            Some((Key::Character(char), *key_code))
                        }
                    } else {
                        eprintln!("Could not process keyboard input {keyboard_input:?}, because it's an empty character.");
                        None
                    }
                }
                KeyboardInput { key_code, .. } => {
                    Some((Key::Code(*key_code), *key_code))
                }
            })
            .map(|(key, key_code)| {
                KeyChange {
                    key,
                change: if ctrl_change || key_states.just_pressed(key_code) {
                    //println!("{:?} just pressed", &key);
                    InputChange::JustPressed
                } else {
                    //println!("{:?} held", &key);
                    InputChange::Held
                },
                }
            }).collect(),
            ctrl
        }
    }

    pub(super) fn process(
        &self,
        commands: &mut Commands,
        key_bindings_fresh_without_ctrl: &Query<(Entity, &KeyBinding<(), ()>)>,
        key_bindings_held_without_ctrl: &Query<(Entity, &KeyBinding<(), Held>)>,
        key_bindings_fresh_with_ctrl: &Query<(Entity, &KeyBinding<Ctrl, ()>)>,
        key_bindings_held_with_ctrl: &Query<(Entity, &KeyBinding<Ctrl, Held>)>,
    ) {
        if self.ctrl.is_some() {
            self.process_inner(
                commands,
                key_bindings_fresh_with_ctrl,
                key_bindings_held_with_ctrl,
            );
        } else {
            self.process_inner(
                commands,
                key_bindings_fresh_without_ctrl,
                key_bindings_held_without_ctrl,
            );
        }
    }

    fn process_inner<C: CtrlState>(
        &self,
        commands: &mut Commands,
        key_bindings_fresh: &Query<(Entity, &KeyBinding<C, ()>)>,
        key_bindings_held: &Query<(Entity, &KeyBinding<C, Held>)>,
    ) {
        for key_change in &self.key_changes {
            // Key bindings that may be held down, don't require checking `key_change.change`.
            let system = Self::matching_system(key_bindings_held, key_change.key);

            let system = system.or_else(|| {
                if key_change.change == InputChange::JustPressed {
                    Self::matching_system(key_bindings_fresh, key_change.key)
                } else {
                    None
                }
            });

            if let Some((entity, system)) = system {
                //println!("System found for {key_change:?}");
                match system.clone() {
                    KeyBindingSystem::Simple(system) => {
                        commands.run_system(system);
                    }
                    KeyBindingSystem::Key(system) => {
                        commands.run_system_with_input(system, key_change.key);
                    }
                    KeyBindingSystem::Entity(system) => {
                        commands.run_system_with_input(system, entity);
                    }
                }
            } else {
                //println!("No system found for {key_change:?}");
            }
        }
    }

    fn matching_system<'a, C: CtrlState, H: HeldState>(
        key_bindings: &'a Query<(Entity, &KeyBinding<C, H>)>,
        key: Key,
    ) -> Option<(Entity, &'a KeyBindingSystem)> {
        key_bindings.iter().find_map(|(entity, binding)| {
            binding
                .matching_system(key)
                .map(|binding| (entity, binding))
        })
    }
}
