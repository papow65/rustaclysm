use crate::prelude::{
    clear_gameplay_events, ActorEvent, ApplicationState, Damage, Healing, StaminaImpact,
};
use bevy::prelude::{App, Events, OnExit, Plugin};

pub(crate) struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        // These resources persist between gameplays.
        app.insert_resource(Events::<ActorEvent<StaminaImpact>>::default())
            .insert_resource(Events::<ActorEvent<Damage>>::default())
            .insert_resource(Events::<ActorEvent<Healing>>::default());

        app.add_systems(
            OnExit(ApplicationState::Gameplay),
            (
                clear_gameplay_events::<ActorEvent<StaminaImpact>>,
                clear_gameplay_events::<ActorEvent<Damage>>,
                clear_gameplay_events::<ActorEvent<Healing>>,
            ),
        );
    }
}
