use crate::application::ApplicationState;
use crate::gameplay::events::systems::{create_event_resources, remove_event_resources};
use bevy::prelude::{App, OnEnter, OnExit, Plugin};

pub(crate) struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ApplicationState::Gameplay), create_event_resources);
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_event_resources);
    }
}
