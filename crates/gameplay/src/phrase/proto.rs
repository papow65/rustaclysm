use crate::{Phrase, Severity, Subject};
use cdda_json_files::Description;

/// Untranslated message to the player
pub(crate) trait ProtoPhrase {
    const SEVERITY: Severity;

    // TODO Add language and formatting options
    fn compose(self) -> Phrase;

    #[must_use]
    fn you(verb: &str) -> Phrase {
        Subject::You.verb(verb, "")
    }
}

impl ProtoPhrase for &Description {
    const SEVERITY: Severity = Severity::Neutral;

    fn compose(self) -> Phrase {
        Phrase::new(&**match self {
            Description::Simple(simple) => simple,
            Description::Complex(complex) => complex.get("str").expect("'str' key"),
        })
    }
}
