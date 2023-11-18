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

        // These resources persist between gameplays.
        app.insert_resource(AmbientLight {
            brightness: 0.2,
            ..AmbientLight::default()
        })
        .insert_resource(ElevationVisibility::default())
        // Loading is slow, so we start loading RelativeSegments immediately.
        .insert_resource(RelativeSegmentsGenerator::new())
        .insert_resource(GameplayCounter::default())
        .insert_resource(Events::<Message>::default())
        .insert_resource(Events::<SpawnSubzoneLevel>::default())
        .insert_resource(Events::<CollapseZoneLevel>::default())
        .insert_resource(Events::<SpawnZoneLevel>::default())
        .insert_resource(Events::<UpdateZoneLevelVisibility>::default())
        .insert_resource(Events::<DespawnZoneLevel>::default())
        .insert_resource(Events::<ActionEvent<Stay>>::default())
        .insert_resource(Events::<ActionEvent<Step>>::default())
        .insert_resource(Events::<ActionEvent<Attack>>::default())
        .insert_resource(Events::<ActionEvent<Smash>>::default())
        .insert_resource(Events::<ActionEvent<Pulp>>::default())
        .insert_resource(Events::<ActionEvent<Close>>::default())
        .insert_resource(Events::<ActionEvent<ItemAction<Wield>>>::default())
        .insert_resource(Events::<ActionEvent<ItemAction<Unwield>>>::default())
        .insert_resource(Events::<ActionEvent<ItemAction<Pickup>>>::default())
        .insert_resource(Events::<ActionEvent<ItemAction<MoveItem>>>::default())
        .insert_resource(Events::<ActionEvent<ItemAction<ExamineItem>>>::default())
        .insert_resource(Events::<ActionEvent<ChangePace>>::default())
        .insert_resource(Events::<ActionEvent<StaminaImpact>>::default())
        .insert_resource(Events::<ActionEvent<Damage>>::default())
        .insert_resource(Events::<ActionEvent<Healing>>::default())
        .insert_resource(Events::<CorpseEvent<Damage>>::default())
        .insert_resource(Events::<TerrainEvent<Damage>>::default())
        .insert_resource(Events::<TerrainEvent<Toggle>>::default());

        create_schedules(app);

        app.add_systems(OnEnter(ApplicationState::Gameplay), startup_systems());
        app.add_systems(Update, update_systems());
        app.add_systems(FixedUpdate, fixed_update_systems());
        app.add_systems(OnExit(ApplicationState::Gameplay), shutdown_systems());
    }
}

fn startup_systems() -> impl IntoSystemConfigs<()> {
    (
        create_independent_resources,
        apply_deferred,
        create_dependent_resources,
        apply_deferred,
        spawn_initial_entities,
        spawn_hud,
        apply_deferred,
    )
        .chain()
}

fn update_systems() -> (impl IntoSystemConfigs<()>, impl IntoSystemConfigs<()>) {
    (
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
        // Loading is slow, so we load RelativeSegments in the background, independent of the current ApplicationState
        load_relative_segments.run_if(not(resource_exists::<RelativeSegments>())),
    )
}

fn fixed_update_systems() -> impl IntoSystemConfigs<()> {
    update_status_fps.run_if(
        in_state(ApplicationState::Gameplay).and_then(resource_exists::<StatusTextSections>()),
    )
}

fn shutdown_systems() -> impl IntoSystemConfigs<()> {
    (
        disable_screen_state,
        apply_deferred,
        despawn::<ApplicationState>,
        remove_gameplay_resources,
        (
            clear_gameplay_events::<Message>,
            clear_gameplay_events::<SpawnSubzoneLevel>,
            clear_gameplay_events::<CollapseZoneLevel>,
            clear_gameplay_events::<SpawnZoneLevel>,
            clear_gameplay_events::<UpdateZoneLevelVisibility>,
            clear_gameplay_events::<DespawnZoneLevel>,
        ),
        (
            clear_gameplay_events::<ActionEvent<Stay>>,
            clear_gameplay_events::<ActionEvent<Step>>,
            clear_gameplay_events::<ActionEvent<Attack>>,
            clear_gameplay_events::<ActionEvent<Smash>>,
            clear_gameplay_events::<ActionEvent<Pulp>>,
            clear_gameplay_events::<ActionEvent<Close>>,
            clear_gameplay_events::<ActionEvent<ItemAction<Wield>>>,
            clear_gameplay_events::<ActionEvent<ItemAction<Unwield>>>,
            clear_gameplay_events::<ActionEvent<ItemAction<Pickup>>>,
            clear_gameplay_events::<ActionEvent<ItemAction<MoveItem>>>,
            clear_gameplay_events::<ActionEvent<ItemAction<ExamineItem>>>,
            clear_gameplay_events::<ActionEvent<ChangePace>>,
            clear_gameplay_events::<ActionEvent<StaminaImpact>>,
            clear_gameplay_events::<ActionEvent<Damage>>,
            clear_gameplay_events::<ActionEvent<Healing>>,
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
