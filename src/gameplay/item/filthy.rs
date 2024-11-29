use crate::gameplay::Fragment;
use bevy::prelude::Component;

#[derive(PartialEq, Debug, Component)]
pub(crate) struct Filthy;

impl Filthy {
    pub(crate) fn fragment() -> Fragment {
        Fragment::filthy("filthy")
    }
}
