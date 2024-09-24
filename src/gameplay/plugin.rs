use crate::application::ApplicationState;
use crate::common::{
    load_async_resource, log_transition_plugin, on_safe_event, AsyncResourceLoader,
};
use crate::gameplay::systems::*;
use crate::gameplay::{
    events::EventPlugin, hud::HudPlugin, update_camera_offset, ActorPlugin, BaseScreenPlugin,
    CameraOffset, CddaPlugin, CharacterScreenPlugin, CraftingScreenPlugin, DeathScreenPlugin,
    DespawnSubzoneLevel, DespawnZoneLevel, ElevationVisibility, FocusPlugin, GameplayCounter,
    GameplayScreenState, Infos, InventoryScreenPlugin, MenuScreenPlugin, RelativeSegments,
    SpawnSubzoneLevel, SpawnZoneLevel, TileLoader, UpdateZoneLevelVisibility,
};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{
    in_state, on_event, resource_exists, resource_exists_and_changed, App, AppExtStates,
    AssetEvent, FixedUpdate, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update,
};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, ecs::schedule::SystemConfigTupleMarker};
use cdda_json_files::{Map, MapMemory, Overmap, OvermapBuffer};

pub(crate) struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameplayScreenState>();
        app.enable_state_scoped_entities::<GameplayScreenState>();

        app.add_plugins((
            ActorPlugin,
            FocusPlugin,
            HudPlugin,
            BaseScreenPlugin,
            CddaPlugin,
            EventPlugin,
            CharacterScreenPlugin,
            CraftingScreenPlugin,
            DeathScreenPlugin,
            InventoryScreenPlugin,
            MenuScreenPlugin,
            FrameTimeDiagnosticsPlugin,
            log_transition_plugin::<GameplayScreenState>,
        ));

        // These resources persist between gameplays.
        app.insert_resource(ElevationVisibility::default())
            .insert_resource(AsyncResourceLoader::<Infos>::default())
            .insert_resource(AsyncResourceLoader::<RelativeSegments>::default())
            .insert_resource(AsyncResourceLoader::<TileLoader>::default())
            .insert_resource(GameplayCounter::default());

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

fn update_systems() -> impl IntoSystemConfigs<(SystemConfigTupleMarker, (), (), (), ())> {
    (
        (
            (
                (
                    handle_overmap_buffer_events.run_if(on_event::<AssetEvent<OvermapBuffer>>()),
                    handle_overmap_events.run_if(on_event::<AssetEvent<Overmap>>()),
                ),
                update_zone_levels_with_missing_assets
                    .run_if(on_safe_event::<AssetEvent<OvermapBuffer>>()),
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
                        .run_if(on_safe_event::<SpawnSubzoneLevel>()),
                    despawn_subzone_levels.run_if(on_safe_event::<DespawnSubzoneLevel>()),
                ),
            )
                .chain(),
            update_visibility.run_if(resource_exists_and_changed::<ElevationVisibility>),
            (
                update_zone_levels,
                (
                    spawn_zone_levels.run_if(on_safe_event::<SpawnZoneLevel>()),
                    update_zone_level_visibility
                        .run_if(on_safe_event::<UpdateZoneLevelVisibility>()),
                    despawn_zone_level.run_if(on_safe_event::<DespawnZoneLevel>()),
                    count_entities.run_if(on_safe_event::<DespawnZoneLevel>()),
                ),
            )
                .chain(),
            manage_gameplay_keyboard_input.run_if(on_event::<KeyboardInput>()),
            update_camera_offset.run_if(resource_exists_and_changed::<CameraOffset>),
        )
            .run_if(in_state(ApplicationState::Gameplay)),
        // Resources that take a while to load, are loaded in the background, independent of the current ApplicationState
        load_async_resource::<RelativeSegments>(),
        load_async_resource::<TileLoader>(),
        load_async_resource::<Infos>(),
    )
}

fn fixed_update_systems() -> impl IntoSystemConfigs<()> {
    (
        (count_assets, count_zones, check_failed_asset_loading),
        #[cfg(feature = "log_archetypes")]
        list_archetypes,
    )
        .chain()
}

fn shutdown_systems() -> impl IntoSystemConfigs<()> {
    (remove_gameplay_resources, increase_counter).chain()
}
