use crate::{AppearanceCache, MeshCaches};
use bevy::prelude::{App, Plugin};
use gameplay_resource::gameplay_resource_plugin;

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            gameplay_resource_plugin::<AppearanceCache>,
            gameplay_resource_plugin::<MeshCaches>,
        ));
    }
}
