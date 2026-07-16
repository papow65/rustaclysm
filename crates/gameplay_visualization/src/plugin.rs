use crate::{Expanded, VisualizationUpdate};
use bevy::prelude::{App, Plugin};
use gameplay_resource::gameplay_resource_plugin;

pub struct GameplayVisualizationPlugin;

impl Plugin for GameplayVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            gameplay_resource_plugin::<Expanded>,
            gameplay_resource_plugin::<VisualizationUpdate>,
        ));
    }
}
