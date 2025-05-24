use crate::Fragment;
use bevy::prelude::Component;
use cdda_json_files::CddaPhase;
use hud::{BLUE_TEXT_COLOR, WARN_TEXT_COLOR};

#[derive(Copy, Clone, Debug, Component)]
#[component(immutable)]
pub(crate) enum Phase {
    Solid,
    Liquid,
    Gas,
}

impl Phase {
    pub(crate) fn suffix(self) -> Option<Fragment> {
        match self {
            Self::Solid => None,
            Self::Liquid => Some(Fragment::colorized("liquid", BLUE_TEXT_COLOR)),
            Self::Gas => Some(Fragment::colorized("gas", WARN_TEXT_COLOR)),
        }
    }
}

impl From<&CddaPhase> for Phase {
    fn from(source: &CddaPhase) -> Self {
        match source {
            CddaPhase::Solid => Self::Solid,
            CddaPhase::Liquid => Self::Liquid,
            CddaPhase::Gas => Self::Gas,
        }
    }
}
