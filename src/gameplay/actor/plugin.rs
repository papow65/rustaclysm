use crate::application::ApplicationState;
use crate::gameplay::actor::behavior::BehaviorPlugin;
use crate::prelude::{
    clear_gameplay_events, ActorEvent, Damage, Healing, PlayerActionState, StaminaImpact,
};
use bevy::prelude::{App, AppExtStates, Events, NextState, OnEnter, OnExit, Plugin, ResMut};

pub(crate) struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BehaviorPlugin);

        app.insert_state(PlayerActionState::Normal);

        // These resources persist between gameplays.
        app.insert_resource(Events::<ActorEvent<StaminaImpact>>::default())
            .insert_resource(Events::<ActorEvent<Damage>>::default())
            .insert_resource(Events::<ActorEvent<Healing>>::default());

        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            |mut next_state: ResMut<NextState<PlayerActionState>>| {
                // TODO match the state from the save
                next_state.set(PlayerActionState::Normal);
            },
        );
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
