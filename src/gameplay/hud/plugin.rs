use super::{
    input::manage_hud_keyboard_input,
    manual::{spawn_manual, update_manual},
    resources::{despawn_hud_resources, spawn_hud_resources},
    sidebar::{spawn_sidebar, update_sidebar_systems, update_status_fps},
};
use crate::prelude::{ApplicationState, GameplayScreenState};
use bevy::{
    input::keyboard::KeyboardInput,
    prelude::{
        in_state, on_event, App, FixedUpdate, IntoSystemConfigs, OnEnter, OnExit, Plugin,
        StateTransitionEvent, Update,
    },
};

/** Plugin for the screen-independent HUD: the manual and the sidebar */
pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            (spawn_hud_resources, (spawn_manual, spawn_sidebar)).chain(),
        );

        app.add_systems(
            Update,
            (
                manage_hud_keyboard_input.run_if(on_event::<KeyboardInput>()),
                update_sidebar_systems(),
                update_manual.run_if(on_event::<StateTransitionEvent<GameplayScreenState>>()),
            )
                .run_if(in_state(ApplicationState::Gameplay)),
        );

        app.add_systems(
            FixedUpdate,
            update_status_fps.run_if(in_state(ApplicationState::Gameplay)),
        );

        app.add_systems(OnExit(ApplicationState::Gameplay), despawn_hud_resources);
    }
}
