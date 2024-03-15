use crate::prelude::{Fragment, Phrase, GOOD_TEXT_COLOR};

#[derive(Clone, Debug)]
pub(crate) enum Subject {
    You,
    Other(Phrase),
}

impl Subject {
    fn phrase(self, second_person: &str, third_person: String) -> Phrase {
        match self {
            Self::You => Phrase::from_fragment(Fragment {
                text: String::from("You"),
                color: Some(GOOD_TEXT_COLOR),
            })
            .add(second_person),
            Self::Other(phrase) => phrase.add(third_person),
        }
    }

    pub(crate) fn verb(self, root: &str, suffix: &str) -> Phrase {
        self.phrase(root, String::from(root) + suffix)
    }

    pub(crate) fn is(self) -> Phrase {
        self.phrase("is", String::from("are"))
    }
}
