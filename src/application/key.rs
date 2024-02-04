use bevy::{
    ecs::system::SystemParam,
    input::{
        keyboard::{Key as LogicalKey, KeyCode, KeyboardInput},
        ButtonState,
    },
    prelude::{ButtonInput, EventReader, Res},
};
use std::fmt;

#[derive(SystemParam)]
pub(crate) struct Keys<'w, 's> {
    keyboard_inputs: EventReader<'w, 's, KeyboardInput>,
    key_states: Res<'w, ButtonInput<KeyCode>>,
}

impl<'w, 's> Keys<'w, 's> {
    /** `Key::Character` is used when possible, unless for numpad1-9. */
    pub(crate) fn combos(&mut self, ctrl: Ctrl) -> impl Iterator<Item = KeyCombo> + '_ {
        if Ctrl::from(&*self.key_states) != ctrl {
            // Prevent incorrect key combos in the next frame
            self.keyboard_inputs.clear();
        }

        self
            .keyboard_inputs
            .read()
            .filter_map(|keyboard_input| match keyboard_input {
                KeyboardInput { state: ButtonState::Released, .. } => None,
                KeyboardInput { key_code, logical_key: LogicalKey::Character(key), .. } if !matches!(key_code, KeyCode::Numpad1 | KeyCode::Numpad2 | KeyCode::Numpad3 | KeyCode::Numpad4 | KeyCode::Numpad5 | KeyCode::Numpad6 | KeyCode::Numpad7 | KeyCode::Numpad8 | KeyCode::Numpad9)=> {
                    let mut chars = key.chars();
                    if let Some(char) = chars.next() {
                        if chars.next().is_some() {
                            eprintln!("Could not process keyboard input {keyboard_input:?}, because it's multiple characters.");
                            None
                        } else {
                            Some((Key::Character(char), key_code))
                        }
                    } else {
                        eprintln!("Could not process keyboard input {keyboard_input:?}, because it's an empty character.");
                        None
                    }
                }
                KeyboardInput { key_code, .. } => {
                    Some((Key::Code(*key_code), key_code))
                }
            })
            .map(move |(key, key_code)| (ctrl, key, key_code))
            .map(|(ctrl, key, key_code)| KeyCombo{ctrl, key, change: InputChange::from((&*self.key_states, *key_code))})
            .map(|combo| {
                println!("{combo:?}");
                combo
            })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Ctrl {
    With,
    Without,
}

impl From<&ButtonInput<KeyCode>> for Ctrl {
    fn from(key_states: &ButtonInput<KeyCode>) -> Self {
        if key_states.pressed(KeyCode::ControlLeft) || key_states.pressed(KeyCode::ControlRight) {
            Self::With
        } else {
            Self::Without
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Character(char),
    Code(KeyCode),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum InputChange {
    JustPressed,
    Held,
}

impl From<(&ButtonInput<KeyCode>, KeyCode)> for InputChange {
    fn from((key_states, key_code): (&ButtonInput<KeyCode>, KeyCode)) -> Self {
        if key_states.just_pressed(KeyCode::ControlLeft)
            || key_states.just_pressed(KeyCode::ControlRight)
            || key_states.just_pressed(key_code)
        {
            Self::JustPressed
        } else {
            Self::Held
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct KeyCombo {
    pub(crate) ctrl: Ctrl,
    pub(crate) key: Key,
    pub(crate) change: InputChange,
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            formatter,
            "{}{:?} ({:?})",
            if self.ctrl == Ctrl::With { "ctrl+" } else { "" },
            self.key,
            self.change
        )
    }
}
