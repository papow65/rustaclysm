use crate::{Fragment, Phrase};

#[must_use]
#[derive(Clone, Debug)]
pub enum Subject {
    You,
    Other(Phrase),
}

impl Subject {
    #[must_use]
    fn phrase(self, second_person: &str, third_person: String) -> Phrase {
        match self {
            Self::You => Phrase::from_fragment(Fragment::you()).hard(second_person),
            Self::Other(phrase) => phrase.hard(third_person),
        }
    }

    #[must_use]
    pub fn verb(self, root: &str, suffix: &str) -> Phrase {
        self.phrase(root, String::from(root) + suffix)
    }

    #[must_use]
    pub fn is(self) -> Phrase {
        self.phrase("are", String::from("is"))
    }

    #[must_use]
    pub fn simple(self, root: &str) -> Phrase {
        self.verb(root, "")
    }
}
