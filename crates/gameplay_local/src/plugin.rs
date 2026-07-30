use crate::GameplayStart;
use application_state::ApplicationState;
use bevy::prelude::{App, FromWorld as _, OnEnter, Plugin, World};
use util::log_resource_change_plugin;

/// This makes make [`GameplayLocal`](`crate::GameplayLocal`) work, by tracking [`GameplayStart`].
pub struct GameplayLocalPlugin;

impl Plugin for GameplayLocalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(log_resource_change_plugin::<GameplayStart>);

        // GameplayStart is kept between gameplay sessions
        app.init_resource::<GameplayStart>();

        app.add_systems(OnEnter(ApplicationState::Gameplay), insert);
    }
}

fn insert(world: &mut World) {
    let new_start = GameplayStart::from_world(world);
    world.insert_resource(new_start); // Overwrites
}
