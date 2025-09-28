use crate::{CurrentlyVisibleBuilder, DebugText, Phrase, PlayerActionState, Positioning, Visible};
use bevy::prelude::{Message, TextColor, TextSpan, info, warn};
use hud::{BAD_TEXT_COLOR, GOOD_TEXT_COLOR, WARN_TEXT_COLOR};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Severity {
    /// For neutral informaion
    Neutral,

    /// For danger to the player character
    Danger,

    /// For actions that can't be performed as instructed
    ImpossibleAction,

    /// For errors caused by the game
    Error,

    /// For positive outcomes for the player character
    Success,
}

impl Severity {
    #[must_use]
    pub(crate) const fn color_override(&self) -> Option<TextColor> {
        match self {
            Self::Neutral => None,
            Self::Danger | Self::ImpossibleAction => Some(WARN_TEXT_COLOR),
            Self::Error => Some(BAD_TEXT_COLOR),
            Self::Success => Some(GOOD_TEXT_COLOR),
        }
    }
}

pub(crate) trait LogMessageTransience: Clone + fmt::Debug + Send + Sync + 'static {}

#[derive(Clone, Debug)]
pub(crate) struct Intransient;

impl LogMessageTransience for Intransient {}

impl LogMessageTransience for PlayerActionState {}

/// `LogMessage` shown to the player
#[derive(Clone, Debug, PartialEq, Eq, Message)]
pub(crate) struct LogMessage<T: LogMessageTransience = Intransient> {
    phrase: Phrase,
    severity: Severity,
    transient_state: T,
}

impl LogMessage<Intransient> {
    pub(crate) const fn new(phrase: Phrase, severity: Severity) -> Self {
        Self {
            phrase,
            severity,
            transient_state: Intransient,
        }
    }
}

impl LogMessage<PlayerActionState> {
    pub(crate) const fn new_transient(
        phrase: Phrase,
        severity: Severity,
        transient_state: PlayerActionState,
    ) -> Self {
        Self {
            phrase,
            severity,
            transient_state,
        }
    }

    pub(crate) const fn transient_state(&self) -> &PlayerActionState {
        &self.transient_state
    }
}

impl<T: LogMessageTransience> LogMessage<T> {
    pub(crate) fn as_text_sections(&self) -> Vec<(TextSpan, TextColor, Option<DebugText>)> {
        self.phrase
            .clone()
            .color_override(self.severity.color_override())
            .as_text_sections()
    }

    fn log(&self, precieved: bool) {
        let suffix = if precieved { "" } else { " (not perceived)" };
        if self.severity == Severity::Error {
            warn!("{}{suffix}", &self.phrase);
        } else {
            info!("{}{suffix}", &self.phrase);
        }
    }

    pub(crate) fn percieved(
        &self,
        currently_visible_builder: &CurrentlyVisibleBuilder,
    ) -> Option<Self> {
        let mut seen = false;
        let mut global = true;
        let mut phrase = self.phrase.clone();

        for fragment in &mut phrase.fragments {
            match fragment.positioning {
                Positioning::Pos(pos) => {
                    if currently_visible_builder
                        .for_player(true)
                        .can_see(pos, None)
                        == Visible::Seen
                    {
                        seen = true;
                    } else {
                        fragment.text = String::from("(unseen)");
                    }
                    global = false;
                }
                Positioning::Player => {
                    seen = true;
                    global = false;
                }
                Positioning::None => {
                    // nothing to do
                }
            }
        }

        let percieved = seen || global;

        self.log(percieved);

        percieved.then_some(Self {
            phrase,
            ..self.clone()
        })
    }
}
