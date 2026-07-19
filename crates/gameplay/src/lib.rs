mod plugin;
mod screens;
mod sidebar;
mod spawn;
mod systems;

pub use self::plugin::GameplayPlugin;

use self::screens::{GameplayScreenState, ScreensPlugin};
use self::sidebar::SidebarPlugin;
use self::spawn::{
    DespawnSubzoneLevel, DespawnZoneLevel, MissingAsset, SpawnPlugin, SpawnSubzoneLevel,
    SpawnZoneLevel, TileSpawner, UpdateZoneLevelVisibility, despawn_systems,
    handle_region_asset_events, handle_zone_levels, spawn_initial_entities, spawn_subzone_levels,
    spawn_subzones_for_camera, update_explored,
};
use self::systems::{
    check_failed_asset_loading, count_assets, count_pos, create_gameplay_key_bindings,
};
