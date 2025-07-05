use crate::{
    Fragment, Intransient, Message, MessageTransience, Phrase, PlayerActionState, Severity, Subject,
};
use bevy::{ecs::system::SystemParam, prelude::EventWriter};

#[derive(SystemParam)]
pub(crate) struct MessageWriter<'w, T: MessageTransience = Intransient> {
    event_writer: EventWriter<'w, Message<T>>,
}

impl<'w, T: MessageTransience> MessageWriter<'w, T> {
    #[must_use]
    pub(crate) const fn subject<'r>(
        &'r mut self,
        subject: Subject,
    ) -> MessageBuilder<'r, 'w, Subject, T>
    where
        'w: 'r,
    {
        MessageBuilder {
            message_writer: self,
            phrase: subject,
        }
    }

    #[must_use]
    pub(crate) fn you<'r>(&'r mut self, verb: &str) -> MessageBuilder<'r, 'w, Phrase, T>
    where
        'w: 'r,
    {
        MessageBuilder {
            message_writer: self,
            phrase: Subject::You.verb(verb, "/"),
        }
    }

    #[must_use]
    pub(crate) fn str<'r, S>(&'r mut self, text: S) -> MessageBuilder<'r, 'w, Phrase, T>
    where
        S: Into<String>,
        'w: 'r,
    {
        MessageBuilder {
            message_writer: self,
            phrase: Phrase::new(text),
        }
    }
}

pub(crate) struct MessageBuilder<'r, 'w, P, T: MessageTransience = Intransient> {
    message_writer: &'r mut MessageWriter<'w, T>,
    phrase: P,
}

impl<'r, 'w, T: MessageTransience> MessageBuilder<'r, 'w, Subject, T> {
    #[must_use]
    pub(crate) fn is(self) -> MessageBuilder<'r, 'w, Phrase, T> {
        self.apply(Subject::is)
    }

    #[must_use]
    pub(crate) fn verb(self, root: &str, suffix: &str) -> MessageBuilder<'r, 'w, Phrase, T> {
        self.apply(|s| s.verb(root, suffix))
    }

    #[must_use]
    pub(crate) fn simple(self, verb: &str) -> MessageBuilder<'r, 'w, Phrase, T> {
        self.verb(verb, "")
    }

    #[must_use]
    pub(crate) fn apply<F>(self, f: F) -> MessageBuilder<'r, 'w, Phrase, T>
    where
        F: FnOnce(Subject) -> Phrase,
    {
        MessageBuilder::<'r, 'w, Phrase, T> {
            message_writer: self.message_writer,
            phrase: f(self.phrase),
        }
    }
}

impl<T: MessageTransience> MessageBuilder<'_, '_, Phrase, T> {
    #[must_use]
    pub(crate) fn soft(mut self, added: impl Into<String>) -> Self {
        self.phrase = self.phrase.soft(added);
        self
    }

    #[must_use]
    pub(crate) fn hard(mut self, added: impl Into<String>) -> Self {
        self.phrase = self.phrase.hard(added);
        self
    }

    #[must_use]
    pub(crate) fn push(mut self, fragment: Fragment) -> Self {
        self.phrase = self.phrase.push(fragment);
        self
    }

    #[must_use]
    pub(crate) fn extend(mut self, fragments: impl IntoIterator<Item = Fragment>) -> Self {
        self.phrase = self.phrase.extend(fragments);
        self
    }
}

impl MessageBuilder<'_, '_, Phrase, Intransient> {
    pub(crate) fn send_info(self) {
        self.send(Severity::Info);
    }

    pub(crate) fn send_warn(self) {
        self.send(Severity::Warn);
    }

    pub(crate) fn send_error(self) {
        self.send(Severity::Error);
    }

    pub(crate) fn send(self, severity: Severity) {
        self.message_writer
            .event_writer
            .write(Message::new(self.phrase, severity));
    }
}

impl MessageBuilder<'_, '_, Phrase, PlayerActionState> {
    pub(crate) fn send_transient(self, severity: Severity, transient_state: PlayerActionState) {
        self.message_writer
            .event_writer
            .write(Message::new_transient(
                self.phrase,
                severity,
                transient_state,
            ));
    }
}
