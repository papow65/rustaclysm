use crate::{Phrase, ProtoPhrase, Severity, Subject};
use units::{Mass, Volume};

#[derive(Debug)]
pub(super) struct CanBearButNeeded {
    pub(super) subject: Subject,
    pub(super) available: Mass,
    pub(super) added: Mass,
}

impl ProtoPhrase for CanBearButNeeded {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        let free_mass = if self.available == Mass::ZERO {
            String::from("no more weight")
        } else {
            format!("only {} more", self.available)
        };
        self.subject
            .simple(format!("can bear {free_mass}, but {} needed", self.added).as_str())
    }
}

#[derive(Debug)]
pub(super) struct CanHoldButNeeded {
    pub(super) subject: Subject,
    pub(super) available: u32,
    pub(super) added: u32,
}

impl ProtoPhrase for CanHoldButNeeded {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        let free_amount = match self.available {
            0 => String::from("no more items"),
            1 => String::from("only one more item"),
            available => format!("only {available} more items"),
        };
        self.subject
            .simple(format!("can hold {free_amount}, but {} needed", self.added).as_str())
    }
}

#[derive(Debug)]
pub(super) struct HasButNeeded {
    pub(super) subject: Subject,
    pub(super) available: Volume,
    pub(super) added: Volume,
}

impl ProtoPhrase for HasButNeeded {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        let free_volume = if self.available == Volume::ZERO {
            String::from("no space left")
        } else {
            format!("only {} available", self.available)
        };
        self.subject
            .simple(format!("has {free_volume}, but {} needed", self.added).as_str())
    }
}
