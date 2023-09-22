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
        .insert_resource(ElevationVisibility::default());

        create_behavior_schedule(app);

        // Loading is slow, so we start loading it immediately.
        // Loading is slow, so we load it in the background as an asset.
        app.add_asset::<RelativeSegments>();
        app.init_asset_loader::<RelativeSegmentsLoader>();
        app.add_systems(
            Update,
            create_relative_segments.run_if(not(resource_exists::<RelativeSegments>())),
        );

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

        app.add_systems(
            Update,
            (
                handle_overmap_buffer_events.run_if(on_event::<AssetEvent<OvermapBuffer>>()),
                handle_overmap_events.run_if(on_event::<AssetEvent<Overmap>>()),
                handle_map_events.run_if(on_event::<AssetEvent<Map>>()),
                (
                    spawn_subzones_for_camera,
                    (
                        spawn_subzone_levels.run_if(
                            resource_exists::<Events<SpawnSubzoneLevel>>()
                                .and_then(on_event::<SpawnSubzoneLevel>()),
                        ),
                        collapse_zone_levels.run_if(
                            resource_exists::<Events<CollapseZoneLevel>>()
                                .and_then(on_event::<CollapseZoneLevel>()),
                        ),
                    ),
                )
                    .chain(),
                update_visualization_on_focus_move
                    .run_if(resource_exists_and_changed::<ElevationVisibility>()),
                (
                    update_collapsed_zone_levels,
                    (
                        spawn_zone_levels.run_if(
                            resource_exists::<Events<SpawnZoneLevel>>()
                                .and_then(on_event::<SpawnZoneLevel>()),
                        ),
                        update_zone_level_visibility.run_if(
                            resource_exists::<Events<UpdateZoneLevelVisibility>>()
                                .and_then(on_event::<UpdateZoneLevelVisibility>()),
                        ),
                    ),
                )
                    .chain(),
                update_camera_offset.run_if(resource_exists_and_changed::<CameraOffset>()),
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

        app.add_systems(
            OnExit(ApplicationState::Gameplay),
            (
                disable_screen_state,
                apply_deferred,
                despawn::<ApplicationState>,
                remove_gameplay_resources,
            )
                .chain(),
        );
    }
}
