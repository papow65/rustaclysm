use crate::{Phrase, ProtoPhrase, Severity};

#[derive(Debug)]
pub(super) struct YouStartTraveling;

impl ProtoPhrase for YouStartTraveling {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
        Self::you("start traveling")
    }
}

#[derive(Debug)]
pub(super) struct YouAreBusy;

impl ProtoPhrase for YouAreBusy {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Self::you("are busy")
    }
}
