use super::{behavior::BehaviorPlugin, player::PlayerActionState};
use crate::prelude::{
    clear_gameplay_events, ActorEvent, ApplicationState, Damage, Healing, StaminaImpact,
};
use bevy::prelude::{App, Commands, Events, OnEnter, OnExit, Plugin};

pub(crate) struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BehaviorPlugin);

        // These resources persist between gameplays.
        app.insert_resource(Events::<ActorEvent<StaminaImpact>>::default())
            .insert_resource(Events::<ActorEvent<Damage>>::default())
            .insert_resource(Events::<ActorEvent<Healing>>::default());

        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            |mut commands: Commands| commands.init_resource::<PlayerActionState>(),
        );

        app.add_systems(
            OnExit(ApplicationState::Gameplay),
            (
                clear_gameplay_events::<ActorEvent<StaminaImpact>>,
                clear_gameplay_events::<ActorEvent<Damage>>,
                clear_gameplay_events::<ActorEvent<Healing>>,
                |mut commands: Commands| commands.remove_resource::<PlayerActionState>(),
            ),
        );
    }
}
