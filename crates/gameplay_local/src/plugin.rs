use crate::GameplayCounter;
use application_state::ApplicationState;
use bevy::prelude::{App, OnExit, Plugin, ResMut};

/// This makes make [`GameplayLocal`](`crate::GameplayLocal`) work, by creating and increasing a [`GameplayCounter`].
pub struct GameplayLocalPlugin;

impl Plugin for GameplayLocalPlugin {
    fn build(&self, app: &mut App) {
        // GameplayCounter is kept between gameplay sessions
        app.insert_resource(GameplayCounter::default());

        app.add_systems(OnExit(ApplicationState::Gameplay), increase_counter);
    }
}

fn increase_counter(mut counter: ResMut<GameplayCounter>) {
    counter.increase();
}
