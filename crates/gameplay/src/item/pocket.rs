use crate::Fragment;
use bevy::prelude::Component;
use cdda_json_files::{CddaPocket, PocketType};

#[derive(Copy, Clone, Debug)]
pub(crate) enum PocketSealing {
    Unsealed,
    Sealed,
}

impl PocketSealing {
    pub(crate) fn suffix(self) -> Option<Fragment> {
        match self {
            Self::Unsealed => None,
            Self::Sealed => Some(Fragment::good("sealed")),
        }
    }
}

#[derive(Debug, Component)]
#[component(immutable)]
pub(crate) struct Pocket {
    pub(crate) type_: PocketType,
    pub(crate) sealing: PocketSealing,
}

impl From<&CddaPocket> for Pocket {
    fn from(source: &CddaPocket) -> Self {
        Self {
            type_: source.pocket_type,
            sealing: if source.sealed {
                PocketSealing::Sealed
            } else {
                PocketSealing::Unsealed
            },
        }
    }
}
