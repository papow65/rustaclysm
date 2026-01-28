use crate::{ProtoLogMessage, Severity};
use text::Phrase;

#[derive(Debug)]
pub(super) struct YouWait;

impl ProtoLogMessage for YouWait {
    const SEVERITY: Severity = Severity::Neutral;

    fn phrase(self) -> Phrase {
        Self::you("wait...")
    }
}
