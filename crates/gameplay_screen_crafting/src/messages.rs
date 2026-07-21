use gameplay_log::{ProtoLogMessage, Severity};
use std::sync::Arc;
use text::Phrase;

#[derive(Debug)]
pub(super) struct YouLackTheMeansToCraft {
    pub(super) recipe: Arc<str>,
}

impl ProtoLogMessage for YouLackTheMeansToCraft {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Self::you("lack")
            .soft("the")
            .hard("means")
            .soft("to")
            .hard("craft")
            .hard(&*self.recipe)
    }
}
