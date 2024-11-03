use crate::gameplay::Fragment;
use crate::hud::FILTHY_COLOR;
use bevy::prelude::Component;

#[derive(PartialEq, Debug, Component)]
pub(crate) struct Filthy;

impl Filthy {
    pub(crate) fn fragment() -> Fragment {
        Fragment::colorized("filthy", FILTHY_COLOR)
    }
}
