use bevy::prelude::Component;
use cdda_json_files::{CddaPocket, PocketType};

#[derive(Debug, Component)]
pub(crate) struct Pocket {
    type_: PocketType,
    sealed: bool,
}

impl From<&CddaPocket> for Pocket {
    fn from(source: &CddaPocket) -> Self {
        Self {
            type_: source.pocket_type,
            sealed: source.sealed,
        }
    }
}
