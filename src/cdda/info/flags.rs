use crate::prelude::MaybeFlat;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(from = "Option<MaybeFlat<String>>")]
#[serde(default)]
pub(crate) struct Flags(Vec<String>);

impl Flags {
    fn contains(&self, value: &str) -> bool {
        self.0.contains(&String::from(value))
    }

    pub(crate) fn aquatic(&self) -> bool {
        self.contains("AQUATIC")
    }

    pub(crate) fn goes_up(&self) -> bool {
        self.contains("GOES_UP") || self.contains("RAMP_UP")
    }

    pub(crate) fn goes_down(&self) -> bool {
        self.contains("GOES_DOWN") || self.contains("RAMP_DOWN")
    }

    pub(crate) fn transparent(&self) -> bool {
        self.contains("TRANSPARENT")
    }

    pub(crate) fn water(&self) -> bool {
        self.contains("SHALLOW_WATER") || self.contains("DEEP_WATER")
    }
}

impl From<Option<MaybeFlat<String>>> for Flags {
    fn from(cdda_flags: Option<MaybeFlat<String>>) -> Self {
        Self(match cdda_flags {
            Some(MaybeFlat::Single(flag)) => vec![flag],
            Some(MaybeFlat::Multi(flags)) => flags,
            None => Vec::new(),
        })
    }
}
