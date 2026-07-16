use crate::{Explored, RelativeSegments};
use bevy::prelude::{App, Plugin};
use gameplay_resource::gameplay_resource_plugin;
use util::async_resource_plugin;

pub struct GameplayPerceptionPlugin;

impl Plugin for GameplayPerceptionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            async_resource_plugin::<RelativeSegments>,
            gameplay_resource_plugin::<Explored>,
        ));
    }
}
