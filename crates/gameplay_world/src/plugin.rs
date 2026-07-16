use crate::ZoneLevelIds;
use bevy::prelude::{App, Plugin};
use gameplay_resource::gameplay_resource_plugin;

pub struct GameplayWorldPlugin;

impl Plugin for GameplayWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((gameplay_resource_plugin::<ZoneLevelIds>,));
    }
}
