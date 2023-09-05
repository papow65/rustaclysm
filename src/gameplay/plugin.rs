use crate::prelude::*;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

pub(crate) struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameplayScreenState>();

        app.add_plugins((
            BaseScreenPlugin,
            CharacterScreenPlugin,
            DeathScreenPlugin,
            InventoryScreenPlugin,
            MenuScreenPlugin,
            FrameTimeDiagnosticsPlugin,
        ));

        // These resources may persist between gameplays.
        app.insert_resource(AmbientLight {
            brightness: 0.2,
            ..AmbientLight::default()
        })
        .insert_resource(RelativeSegments::new())
        .insert_resource(ElevationVisibility::default());

        create_behavior_schedule(app);

        // executed only at gameplay startup
        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            (
                create_independent_resources,
                apply_deferred,
                create_dependent_resources,
                apply_deferred,
                spawn_initial_entities,
                spawn_hud,
                apply_deferred,
            )
                .chain(),
        );

        // executed every frame
        app.add_systems(
            Update,
            (
                (
                    handle_map_events,
                    handle_overmap_buffer_events,
                    update_status_player_action_state
                        .run_if(resource_exists_and_changed::<PlayerActionState>()),
                    update_status_player_wielded.run_if(resource_exists_and_changed::<Timeouts>()),
                    update_status_enemies.run_if(resource_exists_and_changed::<Timeouts>()),
                    update_status_detais.run_if(resource_exists_and_changed::<PlayerActionState>()),
                    update_visualization_on_focus_move
                        .run_if(resource_exists_and_changed::<ElevationVisibility>()),
                    spawn_zones_for_camera
                        .after(update_camera_base)
                        .after(update_camera_offset),
                    update_collapsed_zone_levels
                        .after(update_camera_base)
                        .after(update_camera_offset),
                )
                    .run_if(not(in_state(GameplayScreenState::Death))),
                (
                    update_camera_offset.run_if(resource_exists_and_changed::<CameraOffset>()),
                    update_log,
                    update_status_time.run_if(resource_exists_and_changed::<Timeouts>()),
                    update_status_health,
                    update_status_speed,
                    update_status_stamina,
                ),
            )
                .run_if(in_state(ApplicationState::Gameplay)),
        );

        // executed at fixed interval
        app.add_systems(
            FixedUpdate,
            update_status_fps.run_if(
                in_state(ApplicationState::Gameplay)
                    .and_then(resource_exists::<StatusTextSections>()),
            ),
        );

        // executed only at gameplay shutdown
        app.add_systems(
            OnExit(ApplicationState::Gameplay),
            (
                disable_screen_state,
                apply_deferred,
                despawn_gameplay,
                remove_gameplay_resources,
            )
                .chain(),
        );
    }
}
