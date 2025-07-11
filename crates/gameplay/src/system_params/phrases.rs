use crate::{Phrase, ProtoPhrase, Severity};

#[derive(Debug, PartialEq)]
pub(crate) enum NoStairs {
    Up,
    Down,
}

impl ProtoPhrase for NoStairs {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn compose(self) -> Phrase {
        Phrase::new(match self {
            Self::Up => "No stairs up",
            Self::Down => "No stairs down",
        })
    }
}
