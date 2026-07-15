use crate::{CameraDirection, CameraZoom, UpdateCameraOffset};
use bevy::prelude::{App, Plugin, Update};
use gameplay_resource::gameplay_resource_plugin;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            gameplay_resource_plugin::<CameraDirection>,
            gameplay_resource_plugin::<CameraZoom>,
        ));

        app.configure_sets(Update, UpdateCameraOffset);
    }
}
