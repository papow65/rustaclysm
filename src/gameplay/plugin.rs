use crate::prelude::*;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

pub(crate) struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameplayScreenState>();

        app.add_plugin(BaseScreenPlugin)
            .add_plugin(CharacterScreenPlugin)
            .add_plugin(InventoryScreenPlugin)
            .add_plugin(MenuScreenPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin::default());

        // These resources may persist between gameplays.
        app.insert_resource(AmbientLight {
            brightness: 0.2,
            ..AmbientLight::default()
        })
        .insert_resource(RelativeSegments::new())
        .insert_resource(ElevationVisibility::Shown);

        app.add_schedule(BehaviorSchedule, behavior_schedule());

        // executed only at gameplay startup
        app.add_systems(
            (
                initialize_screen_state,
                create_independent_resources,
                apply_system_buffers,
                create_dependent_resources,
                apply_system_buffers,
                spawn_initial_entities,
                spawn_hud,
                apply_system_buffers,
                finish_loading,
            )
                .chain()
                .in_schedule(OnEnter(ApplicationState::Gameplay)),
        );

        // executed every frame
        app.add_systems(
            (
                update_transforms,
                update_hidden_item_visibility,
                update_cursor_visibility_on_player_change,
                update_visualization_on_item_move,
                update_visualization_on_focus_move,
                update_camera,
            )
                .after(UpdateSet::FlushEffects)
                .in_set(OnUpdate(ApplicationState::Gameplay)),
        );
        app.add_systems(
            (
                handle_map_events,
                update_log,
                update_status_fps,
                update_status_time,
                update_status_health,
                update_status_speed,
                update_status_player_state,
                update_status_detais,
                spawn_zones_for_camera.after(update_camera),
                update_collapsed_zone_levels.after(update_camera),
            )
                .in_set(OnUpdate(ApplicationState::Gameplay)),
        );

        // This system may persist between gameplays.
        app.add_system(check_delay.in_base_set(CoreSet::Last));

        // executed only at gameplay shutdown
        app.add_systems(
            (
                disable_screen_state,
                apply_system_buffers,
                despawn_gameplay,
                remove_gameplay_resources,
            )
                .in_schedule(OnExit(ApplicationState::Gameplay)),
        );
    }
}
