use crate::prelude::*;
use bevy::ecs::schedule::SystemSet;
use bevy::pbr::AmbientLight;
use bevy::prelude::*;

pub(crate) struct RustaclysmPlugin;

impl Plugin for RustaclysmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::default())
            .insert_resource(AmbientLight {
                brightness: 0.2,
                ..AmbientLight::default()
            })
            .insert_resource(ItemInfos::new())
            .insert_resource(Location::default())
            .insert_resource(InstructionQueue::default())
            .insert_resource(RelativeRays::new())
            .insert_resource(TileCaches::default());

        // executed once at startup
        app.add_startup_system_to_stage(StartupStage::PreStartup, create_secondairy_resources)
            .add_startup_system(spawn_hud)
            .add_startup_system(spawn_initial_entities)
            .add_startup_system_set_to_stage(StartupStage::PostStartup, update_systems())
            .add_startup_system_to_stage(StartupStage::PostStartup, maximize_window);

        // executed every frame
        app.add_system_to_stage(CoreStage::PreUpdate, spawn_nearby_overzones)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                spawn_nearby_zones.after(spawn_nearby_overzones),
            )
            .add_system_to_stage(CoreStage::PreUpdate, despawn_far_zones)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                remove_changed_markers
                    .after(spawn_nearby_overzones)
                    .after(spawn_nearby_zones)
                    .after(despawn_far_zones),
            )
            .add_system(manage_game_over)
            .add_system(manage_mouse_input)
            .add_system(manage_keyboard_input)
            .add_system(manage_characters)
            .add_system_set_to_stage(CoreStage::PostUpdate, update_systems())
            /*.add_system_to_stage(CoreStage::Last, check_obstacle_location)
            .add_system_to_stage(CoreStage::Last, check_overlap)
            .add_system_to_stage(CoreStage::Last, check_hierarchy)
            .add_system_to_stage(CoreStage::Last, check_characters)*/
            .add_system_to_stage(CoreStage::Last, check_delay);
    }
}

fn update_systems() -> SystemSet {
    SystemSet::new()
        .with_system(update_location)
        .with_system(update_transforms)
        .with_system(update_visibility_for_hidden_items)
        .with_system(update_cursor_visibility_on_player_change)
        .with_system(update_visualization_on_item_move)
        .with_system(update_visualization_on_player_move)
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
