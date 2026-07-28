use crate::Infos;
use bevy::prelude::{App, Plugin};
use util::async_resource_plugin;

pub struct InfoPlugin;

impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((async_resource_plugin::<Infos>,));
    }
}
