use bevy::input::keyboard::{Key as LogicalKey, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::{ButtonInput, EventReader, KeyCode, Resource};
use either::Either;
use std::iter::empty;

/// This resource is updated every frame, but this rate may differ from the keyboard input from bevy.
/// So a key may not continuously be present in 'held', even when it is held down on the physical keyboard.
#[derive(Debug, Default, Resource)]
pub(crate) struct Keys {
    just_pressed: Vec<Key>,
    key_changes: Vec<KeyChange>,
    ctrl: Ctrl,
}

impl Keys {
    pub(crate) fn update(
        &mut self,
        keyboard_inputs: &mut EventReader<KeyboardInput>,
        key_states: &ButtonInput<KeyCode>,
    ) {
        self.just_pressed.clear();
        self.key_changes.clear();

        let new_ctrl = if key_states.pressed(KeyCode::ControlLeft)
            || key_states.pressed(KeyCode::ControlRight)
        {
            Ctrl::With
        } else {
            Ctrl::Without
        };
        let ctrl_change = self.ctrl != new_ctrl;
        self.ctrl = new_ctrl;

        keyboard_inputs
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
        .for_each(|(key, key_code)| {
            let change = if ctrl_change || key_states.just_pressed(key_code) {
                println!("{:?} just pressed", &key);
                self.just_pressed.push(key);
                InputChange::JustPressed
            } else {
                println!("{:?} held", &key);
                InputChange::Held
            };
            self.key_changes.push(KeyChange {
                key,
                change,
            });
        });
    }

    pub(crate) fn with_ctrl(&self) -> impl Iterator<Item = &KeyChange> + '_ {
        self.all(self.ctrl == Ctrl::With)
    }
    pub(crate) fn without_ctrl(&self) -> impl Iterator<Item = &KeyChange> + '_ {
        self.all(self.ctrl == Ctrl::Without)
    }
    fn all(&self, ctrl_state_ok: bool) -> impl Iterator<Item = &KeyChange> + '_ {
        if ctrl_state_ok {
            Either::Left(self.key_changes.iter())
        } else {
            Either::Right(empty())
        }
    }
    #[expect(unused)]
    pub(crate) fn just_pressed_with_ctrl(&self) -> impl Iterator<Item = &Key> + '_ {
        self.just_pressed(self.ctrl == Ctrl::With)
    }
    pub(crate) fn just_pressed_without_ctrl(&self) -> impl Iterator<Item = &Key> + '_ {
        self.just_pressed(self.ctrl == Ctrl::Without)
    }
    fn just_pressed(&self, ctrl_state_ok: bool) -> impl Iterator<Item = &Key> + '_ {
        if ctrl_state_ok {
            Either::Left(self.just_pressed.iter())
        } else {
            Either::Right(empty())
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Ctrl {
    With,
    #[default]
    Without,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Key {
    Character(char),
    Code(KeyCode),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InputChange {
    JustPressed,
    Held,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeyChange {
    pub(crate) key: Key,
    pub(crate) change: InputChange,
}
