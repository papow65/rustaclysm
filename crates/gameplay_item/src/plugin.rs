use crate::ItemChecksPlugin;
use bevy::prelude::Plugin;

pub struct GameplayItemPlugin;

impl Plugin for GameplayItemPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(ItemChecksPlugin);
    }
}
