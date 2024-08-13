use crate::application::ApplicationState;
use crate::gameplay::events::systems::{clear_event_resources, create_event_resources};
use bevy::prelude::{App, OnExit, Plugin, Startup};

pub(crate) struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_event_resources);
        app.add_systems(OnExit(ApplicationState::Gameplay), clear_event_resources());
    }
}
