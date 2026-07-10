use crate::{Intransient, LogMessageTransience, PosPerceiver, Severity, Transient};
use bevy::prelude::{Message, TextColor, TextSpan, info, warn};
use cdda_json_files::Description;
use hud::DebugText;
use text::{Phrase, Positioning, Subject};

/// `LogMessage` shown to the player
#[derive(Clone, Debug, PartialEq, Eq, Message)]
pub struct LogMessage<T: LogMessageTransience = Intransient> {
    phrase: Phrase,
    severity: Severity,
    transient_state: T,
}

impl<T: Transient> LogMessage<T> {
    pub const fn transient_state(&self) -> &T {
        &self.transient_state
    }
}

impl<T: LogMessageTransience> LogMessage<T> {
    pub fn as_text_sections(&self) -> Vec<(TextSpan, TextColor, Option<DebugText>)> {
        self.phrase
            .clone()
            .color_override(self.severity.color_override())
            .as_text_sections()
    }

    fn log(&self, perceived: bool) {
        let suffix = if perceived { "" } else { " (not perceived)" };
        if self.severity == Severity::Error {
            warn!("{}{suffix}", &self.phrase);
        } else {
            info!("{}{suffix}", &self.phrase);
        }
    }

    /// Perceive this log message based on visibility
    pub fn perceived<P: PosPerceiver>(&self, perceiver: &P) -> Option<Self> {
        let mut seen = false;
        let mut global = true;
        let mut phrase = self.phrase.clone();

        for fragment in &mut phrase.fragments {
            match fragment.positioning {
                Positioning::Pos(pos) => {
                    if perceiver.can_perceive(pos) {
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

        let perceived = seen || global;
        self.log(perceived);

        perceived.then_some(Self {
            phrase,
            ..self.clone()
        })
    }
}

/// Untranslated message to the player
pub trait ProtoLogMessage {
    const SEVERITY: Severity;

    // TODO Add language and formatting options
    fn phrase(self) -> Phrase;

    #[must_use]
    fn you(verb: &str) -> Phrase {
        Subject::You.verb(verb, "")
    }

    fn compose(self) -> LogMessage<Intransient>
    where
        Self: Sized,
    {
        LogMessage {
            phrase: self.phrase(),
            severity: Self::SEVERITY,
            transient_state: Intransient,
        }
    }

    fn compose_transient<T: LogMessageTransience>(self, transient_state: T) -> LogMessage<T>
    where
        Self: Sized,
    {
        LogMessage {
            phrase: self.phrase(),
            severity: Self::SEVERITY,
            transient_state,
        }
    }
}

impl ProtoLogMessage for &Description {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        Phrase::new(&**match self {
            Description::Simple(simple) => simple,
            Description::Complex(complex) => complex.get("str").expect("'str' key"),
        })
    }
}
