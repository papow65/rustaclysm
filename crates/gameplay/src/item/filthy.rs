use bevy::prelude::Component;
use text::Fragment;

#[derive(Clone, Copy, PartialEq, Debug, Component)]
#[component(immutable)]
pub(crate) struct Filthy;

impl Filthy {
    pub(crate) fn fragment() -> Fragment {
        Fragment::filthy("filthy")
    }
}
