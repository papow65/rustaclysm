use crate::{LocationCache, SubzoneLevelCache, ZoneLevelCache};
use bevy::prelude::{App, Plugin};
use gameplay_resource::GameplayResourcePlugin;

pub struct LocationPlugin;

impl Plugin for LocationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameplayResourcePlugin::<LocationCache>::default(),
            GameplayResourcePlugin::<SubzoneLevelCache>::default(),
            GameplayResourcePlugin::<ZoneLevelCache>::default(),
        ));
    }
}
