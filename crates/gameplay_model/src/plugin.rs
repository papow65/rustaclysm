use crate::{AppearanceCache, MeshCaches};
use bevy::prelude::{App, Plugin};
use gameplay_resource::GameplayResourcePlugin;

pub struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameplayResourcePlugin::<AppearanceCache>::default(),
            GameplayResourcePlugin::<MeshCaches>::default(),
        ));
    }
}
