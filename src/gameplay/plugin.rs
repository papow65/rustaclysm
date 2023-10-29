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
        .insert_resource(ElevationVisibility::default())
        // Loading is slow, so we start loading RelativeSegments immediately.
        .insert_resource(RelativeSegmentsGenerator::new());
        app.insert_resource(Events::<Message>::default());
        app.insert_resource(Events::<SpawnSubzoneLevel>::default());
        app.insert_resource(Events::<CollapseZoneLevel>::default());
        app.insert_resource(Events::<SpawnZoneLevel>::default());
        app.insert_resource(Events::<UpdateZoneLevelVisibility>::default());
        app.insert_resource(Events::<DespawnZoneLevel>::default());
        app.insert_resource(Events::<ActorEvent<Stay>>::default());
        app.insert_resource(Events::<ActorEvent<Step>>::default());
        app.insert_resource(Events::<ActorEvent<Attack>>::default());
        app.insert_resource(Events::<ActorEvent<Smash>>::default());
        app.insert_resource(Events::<ActorEvent<Close>>::default());
        app.insert_resource(Events::<ActorEvent<Wield>>::default());
        app.insert_resource(Events::<ActorEvent<Unwield>>::default());
        app.insert_resource(Events::<ActorEvent<Pickup>>::default());
        app.insert_resource(Events::<ActorEvent<MoveItem>>::default());
        app.insert_resource(Events::<ActorEvent<ExamineItem>>::default());
        app.insert_resource(Events::<ActorEvent<ChangePace>>::default());
        app.insert_resource(Events::<ActorEvent<StaminaImpact>>::default());
        app.insert_resource(Events::<ActorEvent<Timeout>>::default());
        app.insert_resource(Events::<ActorEvent<Damage>>::default());
        app.insert_resource(Events::<ActorEvent<Healing>>::default());
        app.insert_resource(Events::<ItemEvent<Damage>>::default());
        app.insert_resource(Events::<TerrainEvent<Toggle>>::default());

        create_schedules(app);

        add_systems(app);
    }
}

fn add_systems(app: &mut App) {
    // Loading is slow, so we load RelativeSegments in the background
    app.add_systems(
        Update,
        load_relative_segments.run_if(not(resource_exists::<RelativeSegments>())),
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
            (
                (
                    handle_overmap_buffer_events.run_if(on_event::<AssetEvent<OvermapAsset>>()),
                    handle_overmap_events.run_if(on_event::<AssetEvent<OvermapAsset>>()),
                ),
                update_zone_levels_with_missing_assets
                    .run_if(on_event::<AssetEvent<OvermapAsset>>()),
            )
                .chain(),
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
                    despawn_zone_level.run_if(
                        resource_exists::<Events<DespawnZoneLevel>>()
                            .and_then(on_event::<DespawnZoneLevel>()),
                    ),
                    count_entities.run_if(
                        resource_exists::<Events<DespawnZoneLevel>>()
                            .and_then(on_event::<DespawnZoneLevel>()),
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
            in_state(ApplicationState::Gameplay).and_then(resource_exists::<StatusTextSections>()),
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
