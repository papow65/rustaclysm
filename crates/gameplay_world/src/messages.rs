use gameplay_log::{ProtoLogMessage, Severity};
use text::Phrase;

#[derive(Debug, PartialEq)]
pub enum NoStairs {
    Up,
    Down,
}

impl ProtoLogMessage for NoStairs {
    const SEVERITY: Severity = Severity::ImpossibleAction;

    fn phrase(self) -> Phrase {
        Phrase::new(match self {
            Self::Up => "No stairs up",
            Self::Down => "No stairs down",
        })
    }
}
