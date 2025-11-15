use crate::{LocationCache, SubzoneLevelCache, ZoneLevelCache};
use bevy::prelude::{App, Plugin};
use gameplay_resource::gameplay_resource_plugin;

pub struct LocationPlugin;

impl Plugin for LocationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            gameplay_resource_plugin::<LocationCache>,
            gameplay_resource_plugin::<SubzoneLevelCache>,
            gameplay_resource_plugin::<ZoneLevelCache>,
        ));
    }
}
