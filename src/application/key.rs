use bevy::{
    ecs::system::SystemParam,
    input::{
        keyboard::{KeyCode, KeyboardInput},
        ButtonState,
    },
    prelude::{ButtonInput, EventReader, ReceivedCharacter, Res},
};
use std::fmt;

#[derive(SystemParam)]
pub(crate) struct Keys<'w, 's> {
    key_events: EventReader<'w, 's, KeyboardInput>,
    character_events: EventReader<'w, 's, ReceivedCharacter>,
    key_codes: Res<'w, ButtonInput<KeyCode>>,
}

impl<'w, 's> Keys<'w, 's> {
    pub(crate) fn combos(&mut self) -> Vec<(ButtonState, KeyCombo)> {
        let ctrl = Ctrl::from(&*self.key_codes);

        // Escape, F-keys, and numpad, with support for modifier keys
        let mut combos = self
            .key_events
            .read()
            .map(|key_event| (key_event.state, KeyCombo::from((ctrl, key_event))))
            .collect::<Vec<_>>();

        // Character keys, with support for special characters
        for combo in self.character_events.read().map(KeyCombo::from) {
            combos.push((ButtonState::Pressed, combo.clone()));
            combos.push((ButtonState::Released, combo));
        }

        combos
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Ctrl {
    With,
    Without,
}

impl From<&ButtonInput<KeyCode>> for Ctrl {
    fn from(keys: &ButtonInput<KeyCode>) -> Self {
        if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
            Self::With
        } else {
            Self::Without
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum KeyCombo {
    KeyCode(Ctrl, KeyCode),

    /** The control key modifies the character, so Ctrl+char is not useful. */
    Character(char),
}

impl From<(Ctrl, &KeyboardInput)> for KeyCombo {
    fn from((ctrl, input): (Ctrl, &KeyboardInput)) -> Self {
        eprintln!(
            "{:?} {:?} {:?}",
            input.key_code, input.logical_key, input.state
        );
        Self::KeyCode(ctrl, input.key_code)
    }
}

impl From<&ReceivedCharacter> for KeyCombo {
    fn from(received_character: &ReceivedCharacter) -> Self {
        // TODO Is this duplicate from logical_key?
        Self::Character(
            received_character
                .char
                .chars()
                .next()
                .expect("Key character should not be on-empty"),
        )
    }
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::KeyCode(ctrl, key_code) => write!(
                formatter,
                "{}{:?}",
                if ctrl == &Ctrl::With { "ctrl+" } else { "" },
                key_code
            ),
            Self::Character(character) => write!(formatter, "{character}"),
        }
    }
}
