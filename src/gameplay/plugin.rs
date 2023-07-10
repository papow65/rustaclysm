use crate::prelude::*;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

pub(crate) struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameplayScreenState>();

        app.add_plugins((
            BaseScreenPlugin,
            CharacterScreenPlugin,
            InventoryScreenPlugin,
            MenuScreenPlugin,
            FrameTimeDiagnosticsPlugin::default(),
        ));

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
                update_camera_offset.run_if(resource_exists_and_changed::<CameraOffset>()),
                handle_map_events,
                handle_overmap_buffer_events,
                update_log,
                update_status_time.run_if(resource_exists_and_changed::<Timeouts>()),
                update_status_health,
                update_status_speed,
                update_status_stamina,
                update_status_player_state
                    .run_if(resource_exists_and_changed::<PlayerActionState>()),
                update_status_detais.run_if(resource_exists_and_changed::<PlayerActionState>()),
                spawn_zones_for_camera
                    .after(update_camera_base)
                    .after(update_camera_offset),
                update_collapsed_zone_levels
                    .after(update_camera_base)
                    .after(update_camera_offset),
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

        // This system may persist between gameplays.
        app.add_systems(Last, check_delay);

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
