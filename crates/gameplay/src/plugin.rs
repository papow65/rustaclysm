use crate::spawn::{
    despawn_systems, handle_region_asset_events, handle_zone_levels, spawn_initial_entities,
    spawn_subzone_levels, spawn_subzones_for_camera, update_explored,
};
use crate::systems::{
    check_failed_asset_loading, count_assets, count_pos, create_gameplay_key_bindings,
    log_archetypes, update_visibility, update_visualization_on_item_move,
};
use crate::{
    ActorPlugin, CameraOffset, GameplayScreenState, PhrasePlugin, RelativeSegments,
    SpawnSubzoneLevel, TimePlugin, TransitionPlugin, VisualizationUpdate, events::EventsPlugin,
    focus::FocusPlugin, item::ItemChecksPlugin, resources::ResourcePlugin, screens::ScreensPlugin,
    sidebar::SidebarPlugin, update_camera_offset,
};
use application_state::ApplicationState;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::{
    App, AppExtStates as _, FixedUpdate, IntoScheduleConfigs as _, OnEnter, Plugin, PostUpdate,
    Update, in_state, on_message, resource_exists, resource_exists_and_changed,
};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, ecs::schedule::ScheduleConfigs};
use gameplay_cdda::{CddaPlugin, Exploration};
use gameplay_local::GameplayLocalPlugin;
use gameplay_location::LocationPlugin;
use gameplay_model::ModelPlugin;
use gameplay_resource::GampelayResourceSet;
use util::log_transition_plugin;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameplayScreenState>();

        app.add_plugins((
            (
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
                LocationPlugin,
                TransitionPlugin,
                ScreensPlugin,
                TimePlugin,
            ),
            FrameTimeDiagnosticsPlugin::default(),
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
            (
                update_explored.run_if(on_message::<Exploration>),
                update_camera_offset.run_if(resource_exists_and_changed::<CameraOffset>),
            ),
            spawn_subzones_for_camera,
            (
                spawn_subzone_levels,
                update_visualization_on_item_move.run_if(resource_exists::<RelativeSegments>),
            )
                .chain()
                .run_if(on_message::<SpawnSubzoneLevel>),
            update_visibility.run_if(resource_exists_and_changed::<VisualizationUpdate>),
        )
            .chain(),
        handle_zone_levels(),
    )
        .run_if(in_state(ApplicationState::Gameplay))
}

fn fixed_update_systems() -> ScheduleConfigs<ScheduleSystem> {
    (
        count_assets,
        count_pos,
        check_failed_asset_loading,
        log_archetypes,
    )
        .into_configs()
}
