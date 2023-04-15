use bevy::{
    ecs::system::SystemParam,
    input::{
        keyboard::{KeyCode, KeyboardInput},
        ButtonState,
    },
    prelude::{EventReader, Input, ReceivedCharacter, Res},
};
use std::fmt;

#[derive(SystemParam)]
pub(crate) struct Keys<'w, 's> {
    key_events: EventReader<'w, 's, KeyboardInput>,
    character_events: EventReader<'w, 's, ReceivedCharacter>,
    keys: Res<'w, Input<KeyCode>>,
}

impl<'w, 's> Keys<'w, 's> {
    pub(crate) fn combos(&mut self) -> Vec<(ButtonState, KeyCombo)> {
        let ctrl = Ctrl::from(&*self.keys);

        // Escape, F-keys, and numpad, with support for modifier keys
        let mut combos = self
            .key_events
            .iter()
            .filter_map(|key_event| {
                KeyCombo::try_from((ctrl, key_event))
                    .ok()
                    .map(|combo| (key_event.state, combo))
            })
            .collect::<Vec<_>>();

        // Character keys, with support for special characters
        for combo in self.character_events.iter().map(KeyCombo::from) {
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

impl From<&Input<KeyCode>> for Ctrl {
    fn from(keys: &Input<KeyCode>) -> Self {
        if keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl) {
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

impl TryFrom<(Ctrl, &KeyboardInput)> for KeyCombo {
    type Error = ();

    fn try_from((ctrl, input): (Ctrl, &KeyboardInput)) -> Result<Self, Self::Error> {
        input
            .key_code
            .map(|key_code| KeyCombo::KeyCode(ctrl, key_code))
            .ok_or(())
    }
}

impl From<&ReceivedCharacter> for KeyCombo {
    fn from(received_character: &ReceivedCharacter) -> Self {
        KeyCombo::Character(received_character.char)
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
