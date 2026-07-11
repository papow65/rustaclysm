use bevy::prelude::Component;
use cdda_json_files::CddaPhase;
use hud::{BLUE_TEXT_COLOR, WARN_TEXT_COLOR};
use text::Fragment;

#[derive(Copy, Clone, Debug, Component)]
#[component(immutable)]
pub enum Phase {
    Solid,
    Liquid,
    Gas,
}

impl Phase {
    #[must_use]
    pub fn suffix(self) -> Option<Fragment> {
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
