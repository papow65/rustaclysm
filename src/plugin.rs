use crate::prelude::*;
use bevy::ecs::schedule::SystemSet;
use bevy::pbr::AmbientLight;
use bevy::prelude::*;

pub(crate) struct RustaclysmPlugin;

impl RustaclysmPlugin {
    const INPUT: &str = "input";
    const EFFECT: &str = "effect";
    const SYNC: &str = "sync";
}

impl Plugin for RustaclysmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::default())
            .insert_resource(AmbientLight {
                brightness: 0.2,
                ..AmbientLight::default()
            })
            .insert_resource(Infos::new())
            .insert_resource(Location::default())
            .insert_resource(SubzoneLevelEntities::default())
            .insert_resource(ZoneLevelEntities::default())
            .insert_resource(InstructionQueue::default())
            .insert_resource(RelativeSegments::new())
            .insert_resource(TileCaches::default());

        // executed once at startup
        app.add_startup_system_to_stage(StartupStage::PreStartup, create_secondairy_resources)
            .add_startup_system(spawn_hud)
            .add_startup_system(spawn_initial_entities)
            .add_startup_system_set_to_stage(StartupStage::PostStartup, sync_systems())
            .add_startup_system_to_stage(StartupStage::PostStartup, maximize_window);

        app.add_stage_after(CoreStage::PreUpdate, Self::INPUT, SystemStage::parallel());
        app.add_stage_after(CoreStage::Update, Self::EFFECT, SystemStage::parallel());
        app.add_stage_after(Self::EFFECT, Self::SYNC, SystemStage::parallel());

        // executed every frame
        app.add_system_set_to_stage(CoreStage::PreUpdate, zone_level_systems())
            .add_system_to_stage(Self::INPUT, manage_mouse_input)
            .add_system_to_stage(Self::INPUT, manage_keyboard_input)
            .add_system(manage_characters)
            .add_system_to_stage(Self::EFFECT, manage_game_over)
            .add_system_to_stage(Self::EFFECT, toggle_doors)
            .add_system_set_to_stage(Self::SYNC, sync_systems())
            .add_system_set_to_stage(CoreStage::Last, check_systems());
    }
}

fn sync_systems() -> SystemSet {
    SystemSet::new()
        .with_system(update_location)
        .with_system(update_subzone_level_entities)
        .with_system(update_zone_level_entities)
        .with_system(update_transforms)
        .with_system(update_hidden_item_visibility)
        .with_system(update_cursor_visibility_on_player_change)
        .with_system(update_visualization_on_item_move)
        .with_system(update_visualization_on_focus_move)
        .with_system(update_damaged_characters)
        .with_system(update_damaged_items)
        .with_system(update_camera)
        .with_system(update_log)
        .with_system(update_status_fps)
        .with_system(update_status_time)
        .with_system(update_status_health)
        .with_system(update_status_speed)
        .with_system(update_status_player_state)
        .with_system(update_status_detais)
}

fn check_systems() -> SystemSet {
    SystemSet::new()
        //.with_system(check_obstacle_location)
        //.with_system(check_overla)
        //.with_system(check_hierarchy)
        //.with_system(check_characters)
        //.with_system(check_zone_levels)
        .with_system(check_delay)
}

fn zone_level_systems() -> SystemSet {
    SystemSet::new()
        .with_system(spawn_zones_for_camera)
        .with_system(update_collapsed_zone_levels)
}

trait AppBuilderExt {
    fn add_startup_system_set_to_stage(
        &mut self,
        startup_stage: StartupStage,
        system_set: SystemSet,
    ) -> &mut Self;
}

impl AppBuilderExt for App {
    fn add_startup_system_set_to_stage(
        &mut self,
        startup_stage: StartupStage,
        system_set: SystemSet,
    ) -> &mut Self {
        self.schedule
            .stage(StartupStage::PostStartup, |schedule: &mut Schedule| {
                schedule.add_system_set_to_stage(startup_stage, system_set)
            });
        self
    }
}
