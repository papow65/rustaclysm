use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct BaseScreenPlugin;

impl Plugin for BaseScreenPlugin {
    fn build(&self, app: &mut App) {
        // executed only at startup
        app.add_systems((create_base_resources,).in_schedule(OnEnter(GameplayScreenState::Base)));

        // executed every frame
        app.add_system(
            manage_mouse_input
                .before(update_camera)
                .run_if(in_state(ApplicationState::Gameplay))
                .run_if(in_state(GameplayScreenState::Base)),
        )
        .add_systems(
            (manage_keyboard_input, run_behavior_schedule)
                .chain()
                .in_set(OnUpdate(ApplicationState::Gameplay))
                .in_set(OnUpdate(GameplayScreenState::Base)),
        );

        // executed only at shutdown
        app.add_systems((remove_base_resources,).in_schedule(OnExit(ApplicationState::Gameplay)));
    }
}
