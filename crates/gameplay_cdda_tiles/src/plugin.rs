use crate::TileLoader;
use bevy::prelude::{App, Plugin};
use util::async_resource_plugin;

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((async_resource_plugin::<TileLoader>,));
    }
}
