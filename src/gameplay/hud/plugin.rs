use super::{
    despawn_hud_resources, manage_hud_keyboard_input, spawn_hud_resources, spawn_manual,
    spawn_sidebar, update_log, update_status_detais, update_status_enemies, update_status_fps,
    update_status_health, update_status_player_action_state, update_status_player_wielded,
    update_status_speed, update_status_stamina, update_status_time,
};
use crate::prelude::{
    ApplicationState, Message, PlayerActionState, RefreshAfterBehavior, Timeouts,
};
use bevy::prelude::{
    in_state, on_event, resource_exists_and_changed, App, Condition, FixedUpdate,
    IntoSystemConfigs, OnEnter, OnExit, Plugin, Update,
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
                manage_hud_keyboard_input,
                (
                    // sidebar components, in order:
                    // (fps is handled elsewhere)
                    update_status_time.run_if(resource_exists_and_changed::<Timeouts>),
                    update_status_health,
                    update_status_stamina,
                    update_status_speed,
                    update_status_player_action_state
                        .run_if(resource_exists_and_changed::<PlayerActionState>),
                    update_status_player_wielded.run_if(resource_exists_and_changed::<Timeouts>),
                    update_status_enemies.run_if(resource_exists_and_changed::<Timeouts>),
                    update_status_detais.run_if(resource_exists_and_changed::<PlayerActionState>),
                    update_log.run_if(on_event::<Message>()),
                )
                    .run_if(
                        in_state(ApplicationState::Gameplay)
                            .and_then(on_event::<RefreshAfterBehavior>()),
                    ),
            ),
        );

        app.add_systems(
            FixedUpdate,
            update_status_fps.run_if(in_state(ApplicationState::Gameplay)),
        );

        app.add_systems(OnExit(ApplicationState::Gameplay), despawn_hud_resources);
    }
}
