use crate::prelude::*;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

pub(crate) struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(GameplayScreenState::Inapplicable);

        app.add_plugins((
            ActorPlugin,
            BehaviorPlugin,
            HudPlugin,
            BaseScreenPlugin,
            CharacterScreenPlugin,
            DeathScreenPlugin,
            InventoryScreenPlugin,
            MenuScreenPlugin,
            FrameTimeDiagnosticsPlugin,
        ));

        // These resources persist between gameplays.
        app.insert_resource(ElevationVisibility::default())
            // Loading is slow, so we start loading RelativeSegments immediately.
            .insert_resource(RelativeSegmentsGenerator::new())
            .insert_resource(GameplayCounter::default())
            .insert_resource(Events::<Message>::default())
            .insert_resource(Events::<SpawnSubzoneLevel>::default())
            .insert_resource(Events::<DespawnSubzoneLevel>::default())
            .insert_resource(Events::<SpawnZoneLevel>::default())
            .insert_resource(Events::<UpdateZoneLevelVisibility>::default())
            .insert_resource(Events::<DespawnZoneLevel>::default())
            .insert_resource(Events::<CorpseEvent<Damage>>::default())
            .insert_resource(Events::<TerrainEvent<Damage>>::default())
            .insert_resource(Events::<TerrainEvent<Toggle>>::default())
            .insert_resource(Events::<RefreshAfterBehavior>::default());

        app.add_systems(OnEnter(ApplicationState::Gameplay), startup_systems());
        app.add_systems(Update, update_systems());
        app.add_systems(FixedUpdate, fixed_update_systems());
        app.add_systems(OnExit(ApplicationState::Gameplay), shutdown_systems());
    }
}

fn startup_systems() -> impl IntoSystemConfigs<()> {
    (
        create_independent_resources,
        create_dependent_resources,
        spawn_initial_entities,
    )
        .chain()
}

fn update_systems() -> (impl IntoSystemConfigs<()>, impl IntoSystemConfigs<()>) {
    (
        (
            (
                (
                    handle_overmap_buffer_events.run_if(on_event::<AssetEvent<OvermapBuffer>>()),
                    handle_overmap_events.run_if(on_event::<AssetEvent<Overmap>>()),
                ),
                update_zone_levels_with_missing_assets.run_if(
                    on_event::<AssetEvent<Overmap>>()
                        .or_else(on_event::<AssetEvent<OvermapBuffer>>()),
                ),
            )
                .chain(),
            handle_map_events.run_if(on_event::<AssetEvent<Map>>()),
            handle_map_memory_events.run_if(on_event::<AssetEvent<MapMemory>>()),
            (
                spawn_subzones_for_camera,
                (
                    (
                        spawn_subzone_levels,
                        update_visualization_on_item_move
                            .run_if(resource_exists::<RelativeSegments>),
                    )
                        .chain()
                        .run_if(
                            resource_exists::<Events<SpawnSubzoneLevel>>
                                .and_then(on_event::<SpawnSubzoneLevel>()),
                        ),
                    despawn_subzone_levels.run_if(
                        resource_exists::<Events<DespawnSubzoneLevel>>
                            .and_then(on_event::<DespawnSubzoneLevel>()),
                    ),
                ),
            )
                .chain(),
            update_visualization_on_focus_move
                .run_if(resource_exists_and_changed::<ElevationVisibility>),
            (
                update_zone_levels,
                (
                    spawn_zone_levels.run_if(
                        resource_exists::<Events<SpawnZoneLevel>>
                            .and_then(on_event::<SpawnZoneLevel>()),
                    ),
                    update_zone_level_visibility.run_if(
                        resource_exists::<Events<UpdateZoneLevelVisibility>>
                            .and_then(on_event::<UpdateZoneLevelVisibility>()),
                    ),
                    despawn_zone_level.run_if(
                        resource_exists::<Events<DespawnZoneLevel>>
                            .and_then(on_event::<DespawnZoneLevel>()),
                    ),
                    count_entities.run_if(
                        resource_exists::<Events<DespawnZoneLevel>>
                            .and_then(on_event::<DespawnZoneLevel>()),
                    ),
                ),
            )
                .chain(),
            update_camera_offset.run_if(resource_exists_and_changed::<CameraOffset>),
        )
            .run_if(in_state(ApplicationState::Gameplay)),
        // Loading is slow, so we load RelativeSegments in the background, independent of the current ApplicationState
        load_relative_segments.run_if(not(resource_exists::<RelativeSegments>)),
    )
}

fn fixed_update_systems() -> impl IntoSystemConfigs<()> {
    ((
        #[cfg(debug_assertions)]
        (count_assets, count_zones),
        #[cfg(feature = "log_archetypes")]
        list_archetypes,
    ),)
        .chain()
}

fn shutdown_systems() -> impl IntoSystemConfigs<()> {
    (
        disable_screen_state,
        despawn::<ApplicationState>,
        remove_gameplay_resources,
        (
            clear_gameplay_events::<Message>,
            clear_gameplay_events::<SpawnSubzoneLevel>,
            clear_gameplay_events::<DespawnSubzoneLevel>,
            clear_gameplay_events::<SpawnZoneLevel>,
            clear_gameplay_events::<UpdateZoneLevelVisibility>,
            clear_gameplay_events::<DespawnZoneLevel>,
        ),
        (
            clear_gameplay_events::<CorpseEvent<Damage>>,
            clear_gameplay_events::<TerrainEvent<Damage>>,
            clear_gameplay_events::<TerrainEvent<Toggle>>,
        ),
        increase_counter,
    )
        .chain()
}
