use crate::{Fragment, Message, Phrase, Severity, Subject};
use bevy::ecs::system::SystemParam;
use bevy::prelude::EventWriter;

#[derive(SystemParam)]
pub(crate) struct MessageWriter<'w> {
    event_writer: EventWriter<'w, Message>,
}

impl<'w> MessageWriter<'w> {
    #[must_use]
    pub(crate) const fn subject<'r>(
        &'r mut self,
        subject: Subject,
    ) -> MessageBuilder<'r, 'w, Subject>
    where
        'w: 'r,
    {
        MessageBuilder {
            message_writer: self,
            phrase: subject,
        }
    }

    #[must_use]
    pub(crate) fn you<'r>(&'r mut self, verb: &str) -> MessageBuilder<'r, 'w, Phrase>
    where
        'w: 'r,
    {
        MessageBuilder {
            message_writer: self,
            phrase: Subject::You.verb(verb, "/"),
        }
    }

    #[must_use]
    pub(crate) fn str<'r, S>(&'r mut self, text: S) -> MessageBuilder<'r, 'w, Phrase>
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

pub(crate) struct MessageBuilder<'r, 'w, T> {
    message_writer: &'r mut MessageWriter<'w>,
    phrase: T,
}

impl<'r, 'w> MessageBuilder<'r, 'w, Subject> {
    #[must_use]
    pub(crate) fn is(self) -> MessageBuilder<'r, 'w, Phrase> {
        self.apply(Subject::is)
    }

    #[must_use]
    pub(crate) fn verb(self, root: &str, suffix: &str) -> MessageBuilder<'r, 'w, Phrase> {
        self.apply(|s| s.verb(root, suffix))
    }

    #[must_use]
    pub(crate) fn simple(self, verb: &str) -> MessageBuilder<'r, 'w, Phrase> {
        self.verb(verb, "")
    }

    #[must_use]
    pub(crate) fn apply<F>(self, f: F) -> MessageBuilder<'r, 'w, Phrase>
    where
        F: FnOnce(Subject) -> Phrase,
    {
        MessageBuilder::<'r, 'w, Phrase> {
            message_writer: self.message_writer,
            phrase: f(self.phrase),
        }
    }
}

impl MessageBuilder<'_, '_, Phrase> {
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

    pub(crate) fn send_info(self) {
        self.send(Severity::Info, false);
    }

    pub(crate) fn send_warn(self) {
        self.send(Severity::Warn, false);
    }

    pub(crate) fn send_error(self) {
        self.send(Severity::Error, false);
    }

    pub(crate) fn send(self, severity: Severity, transient: bool) {
        self.message_writer.event_writer.write(Message {
            phrase: self.phrase,
            severity,
            transient,
        });
    }
}
