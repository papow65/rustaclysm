use crate::GameplayResourcePlugin;
use crate::models::resources::{AppearanceCache, MeshCaches};
use bevy::prelude::{App, Plugin};

pub(crate) struct ModelPlugin;

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameplayResourcePlugin::<AppearanceCache>::default(),
            GameplayResourcePlugin::<MeshCaches>::default(),
        ));
    }
}
