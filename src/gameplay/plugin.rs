use crate::application::ApplicationState;
use crate::gameplay::spawn::{
    despawn_subzone_levels, despawn_zone_level, handle_map_events, handle_map_memory_events,
    handle_overmap_buffer_events, handle_overmap_events, spawn_initial_entities,
    spawn_subzone_levels, spawn_subzones_for_camera, spawn_zone_levels, update_explored,
    update_zone_level_visibility, update_zone_levels, update_zone_levels_with_missing_assets,
};
use crate::gameplay::systems::{
    check_failed_asset_loading, count_assets, count_pos, create_gameplay_key_bindings,
    update_visibility, update_visualization_on_item_move,
};
use crate::gameplay::{
    ActorPlugin, CameraOffset, CddaPlugin, DespawnSubzoneLevel, DespawnZoneLevel, Exploration,
    GameplayScreenState, MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset, PhrasePlugin,
    RelativeSegments, SpawnSubzoneLevel, SpawnZoneLevel, UpdateZoneLevelVisibility,
    VisualizationUpdate, update_camera_offset,
};
use crate::gameplay::{events::EventsPlugin, focus::FocusPlugin, item::ItemChecksPlugin};
use crate::gameplay::{models::ModelPlugin, sidebar::SidebarPlugin, time::TimePlugin};
use crate::gameplay::{
    resources::ResourcePlugin, scope::GameplayLocalPlugin, screens::ScreensPlugin,
};
use bevy::prelude::{
    App, AppExtStates as _, AssetEvent, FixedUpdate, IntoSystemConfigs as _, Last, OnEnter, Plugin,
    Update, in_state, on_event, resource_exists, resource_exists_and_changed,
};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, ecs::schedule::SystemConfigs};
use util::log_transition_plugin;

pub(crate) struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameplayScreenState>();
        app.enable_state_scoped_entities::<GameplayScreenState>();

        app.add_plugins((
            ActorPlugin,
            FocusPlugin,
            SidebarPlugin,
            CddaPlugin,
            EventsPlugin,
            ItemChecksPlugin,
            ModelPlugin,
            PhrasePlugin,
            ResourcePlugin,
            GameplayLocalPlugin,
            ScreensPlugin,
            TimePlugin,
            FrameTimeDiagnosticsPlugin,
            log_transition_plugin::<GameplayScreenState>,
        ));

        app.add_systems(OnEnter(ApplicationState::Gameplay), startup_systems());
        app.add_systems(Update, update_systems());
        app.add_systems(FixedUpdate, fixed_update_systems());
        app.add_systems(Last, despawn_systems());
    }
}

fn startup_systems() -> SystemConfigs {
    (spawn_initial_entities, create_gameplay_key_bindings).into_configs()
}

fn update_systems() -> SystemConfigs {
    (
        (
            (
                handle_overmap_buffer_events.run_if(on_event::<AssetEvent<OvermapBufferAsset>>),
                handle_overmap_events.run_if(on_event::<AssetEvent<OvermapAsset>>),
            ),
            update_zone_levels_with_missing_assets
                .run_if(on_event::<AssetEvent<OvermapBufferAsset>>),
        )
            .chain(),
        handle_map_events.run_if(on_event::<AssetEvent<MapAsset>>),
        handle_map_memory_events.run_if(on_event::<AssetEvent<MapMemoryAsset>>),
        (
            (
                update_explored.run_if(on_event::<Exploration>),
                update_camera_offset.run_if(resource_exists_and_changed::<CameraOffset>),
            ),
            spawn_subzones_for_camera,
            (
                spawn_subzone_levels.run_if(on_event::<SpawnSubzoneLevel>),
                update_visualization_on_item_move.run_if(resource_exists::<RelativeSegments>),
            )
                .chain()
                .run_if(on_event::<SpawnSubzoneLevel>),
            update_visibility.run_if(resource_exists_and_changed::<VisualizationUpdate>),
        )
            .chain(),
        (
            update_zone_levels,
            (
                spawn_zone_levels.run_if(on_event::<SpawnZoneLevel>),
                update_zone_level_visibility.run_if(on_event::<UpdateZoneLevelVisibility>),
            ),
        )
            .chain(),
    )
        .run_if(in_state(ApplicationState::Gameplay))
}

fn fixed_update_systems() -> SystemConfigs {
    (
        (count_assets, count_pos, check_failed_asset_loading),
        #[cfg(feature = "log_archetypes")]
        list_archetypes,
    )
        .into_configs()
}

/// This should run last, to prevent Bevy crashing on despawned entities being modified.
fn despawn_systems() -> SystemConfigs {
    (
        despawn_subzone_levels.run_if(on_event::<DespawnSubzoneLevel>),
        despawn_zone_level.run_if(on_event::<DespawnZoneLevel>),
    )
        .run_if(in_state(ApplicationState::Gameplay))
}
