use crate::{Phrase, ProtoPhrase, Severity};

#[derive(Debug)]
pub(super) struct YouStartTraveling;

impl ProtoPhrase for YouStartTraveling {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
        Self::you("start traveling")
    }
}
