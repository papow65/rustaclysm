use crate::{
    ScreensPlugin, check_failed_asset_loading, count_assets, count_pos,
    create_gameplay_key_bindings,
};
use application_state::ApplicationState;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::{
    App, AppExtStates as _, FixedUpdate, IntoScheduleConfigs as _, OnEnter, Plugin, PostUpdate,
    Update, in_state, on_message, resource_exists, resource_exists_and_changed,
};
use gameplay_behavior::BehaviorLoopPlugin;
use gameplay_camera::UpdateCameraOffset;
use gameplay_cdda::{CddaPlugin, Exploration};
use gameplay_character::CharacterPlugin;
use gameplay_focus::{FocusPlugin, OnFocusChange};
use gameplay_item::GameplayItemPlugin;
use gameplay_local::GameplayLocalPlugin;
use gameplay_location::LocationPlugin;
use gameplay_log::LogPlugin;
use gameplay_model::ModelPlugin;
use gameplay_perception::{GameplayPerceptionPlugin, RelativeSegments};
use gameplay_player::PlayerPlugin;
use gameplay_resource::GampelayResourceSet;
use gameplay_screen_state::GameplayScreenState;
use gameplay_sidebar::SidebarPlugin;
use gameplay_spawn::{
    SpawnPlugin, despawn_systems, handle_region_asset_events, handle_zone_levels,
    spawn_initial_entities, spawn_subzone_levels, spawn_subzones_for_camera, update_explored,
};
use gameplay_terrain::TerrainPlugin;
use gameplay_time::TimePlugin;
use gameplay_transition::TransitionPlugin;
use gameplay_visualization::{
    GameplayVisualizationPlugin, VisualizationUpdate, update_visibility,
    update_visualization_on_item_move,
};
use gameplay_world::GameplayWorldPlugin;
use util::log_transition_plugin;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameplayScreenState>();

        app.add_plugins((
            (
                BehaviorLoopPlugin,
                CharacterPlugin,
                FocusPlugin,
                CddaPlugin,
                GameplayItemPlugin,
                GameplayLocalPlugin,
                GameplayPerceptionPlugin,
                GameplayVisualizationPlugin,
                GameplayWorldPlugin,
                LocationPlugin,
                LogPlugin,
                ModelPlugin,
                PlayerPlugin,
                (
                    SpawnPlugin,
                    ScreensPlugin,
                    SidebarPlugin,
                    TerrainPlugin,
                    TimePlugin,
                    TransitionPlugin,
                ),
            ),
            log_transition_plugin::<GameplayScreenState>,
        ));

        app.add_systems(OnEnter(ApplicationState::Gameplay), startup_systems());
        app.add_systems(Update, update_systems());
        app.add_systems(FixedUpdate, fixed_update_systems());
        app.add_systems(PostUpdate, despawn_systems());
    }
}

fn startup_systems() -> ScheduleConfigs<ScheduleSystem> {
    (
        spawn_initial_entities.after(GampelayResourceSet),
        create_gameplay_key_bindings,
    )
        .into_configs()
}

fn update_systems() -> ScheduleConfigs<ScheduleSystem> {
    (
        handle_region_asset_events(),
        (
            (update_explored.run_if(on_message::<Exploration>),),
            spawn_subzones_for_camera.after(UpdateCameraOffset),
            (
                spawn_subzone_levels,
                update_visualization_on_item_move.run_if(resource_exists::<RelativeSegments>),
            )
                .chain(),
            update_visibility.run_if(resource_exists_and_changed::<VisualizationUpdate>),
        )
            .chain(),
        handle_zone_levels(),
        update_visibility.in_set(OnFocusChange),
    )
        .run_if(in_state(ApplicationState::Gameplay))
}

fn fixed_update_systems() -> ScheduleConfigs<ScheduleSystem> {
    (count_assets, count_pos, check_failed_asset_loading).into_configs()
}
