use crate::{Phrase, ProtoPhrase, Severity};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct YouLackTheMeansToCraft {
    pub(super) recipe: Arc<str>,
}

impl ProtoPhrase for YouLackTheMeansToCraft {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Self::you("lack")
            .soft("the")
            .hard("means")
            .soft("to")
            .hard("craft")
            .hard(&*self.recipe)
    }
}
