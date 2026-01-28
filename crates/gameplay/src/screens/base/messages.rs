use crate::{ProtoLogMessage, Severity};
use text::Phrase;

#[derive(Debug)]
pub(super) struct YouStartTraveling;

impl ProtoLogMessage for YouStartTraveling {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        Self::you("start traveling")
    }
}

#[derive(Debug)]
pub(super) struct YouAreBusy;

impl ProtoLogMessage for YouAreBusy {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Self::you("are busy")
    }
}
