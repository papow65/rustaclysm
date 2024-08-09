use crate::application::ApplicationState;
use crate::gameplay::actor::behavior::BehaviorPlugin;
use crate::gameplay::{
    clear_gameplay_events, ActorEvent, Damage, Healing, PlayerActionState, StaminaImpact,
};
use bevy::prelude::{App, AppExtStates, Events, OnExit, Plugin};

pub(crate) struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BehaviorPlugin);

        app.add_sub_state::<PlayerActionState>();

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
