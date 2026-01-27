use crate::{Phrase, ProtoPhrase, Severity};

#[derive(Debug)]
pub(super) struct YouWait;

impl ProtoPhrase for YouWait {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
        Self::you("wait...")
    }
}
