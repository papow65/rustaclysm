use crate::gameplay::{Fragment, Phrase};

#[derive(Clone, Debug)]
pub(crate) enum Subject {
    You,
    Other(Phrase),
}

impl Subject {
    fn phrase(self, second_person: &str, third_person: String) -> Phrase {
        match self {
            Self::You => Phrase::from_fragment(Fragment::you()).add(second_person),
            Self::Other(phrase) => phrase.add(third_person),
        }
    }

    pub(crate) fn verb(self, root: &str, suffix: &str) -> Phrase {
        self.phrase(root, String::from(root) + suffix)
    }

    pub(crate) fn is(self) -> Phrase {
        self.phrase("are", String::from("is"))
    }
}
