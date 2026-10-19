use crate::RefreshScope;
use bevy::prelude::{App, Plugin};
use gameplay_resource::gameplay_resource_plugin;

pub struct GameplayRefreshPlugin;

impl Plugin for GameplayRefreshPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(gameplay_resource_plugin::<RefreshScope>);
    }
}
