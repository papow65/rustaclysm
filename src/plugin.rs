use bevy::ecs::schedule::SystemSet;
use bevy::pbr::AmbientLight;
use bevy::prelude::*;

use super::resources::{Instructions, Location, RelativeRays, Timeouts};
use super::systems::*;

pub struct RustaclysmPlugin;

impl Plugin for RustaclysmPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Msaa { samples: 4 }) // early, to prevent a vulkan error
            .insert_resource(WindowDescriptor {
                vsync: false, // much better responsiveness
                ..WindowDescriptor::default()
            })
            .insert_resource(AmbientLight {
                brightness: 0.2,
                ..AmbientLight::default()
            })
            .insert_resource(Location::new())
            .insert_resource(RelativeRays::new())
            .insert_resource(Instructions::new())
            .insert_resource(Timeouts::new());

        // executed once at startup
        app.add_startup_system(add_entities.system())
            .add_startup_system_set_to_stage(StartupStage::PostStartup, update_systems());

        // executed every frame
        app.add_system(manage_game_over.system())
            .add_system(manage_status.system())
            .add_system(manage_mouse_input.system())
            .add_system(manage_keyboard_input.system())
            .add_system(manage_characters.system())
            .add_system_set_to_stage(CoreStage::PostUpdate, update_systems())
            /*.add_system_to_stage(CoreStage::Last, check_obstacle_location.system())
            .add_system_to_stage(CoreStage::Last, check_overlap.system())
            .add_system_to_stage(CoreStage::Last, check_hierarchy.system())
            .add_system_to_stage(CoreStage::Last, check_characters.system())*/
            .add_system_to_stage(CoreStage::Last, check_delay.system());
    }
}

fn update_systems() -> SystemSet {
    SystemSet::new()
        .with_system(update_location.system())
        .with_system(update_transforms.system())
        .with_system(update_visible_for_hidden_items.system())
        .with_system(update_visible_on_item_y_change.system())
        .with_system(update_visible_on_player_y_change.system())
        .with_system(update_material_on_item_move.system())
        .with_system(update_material_on_player_move.system())
        .with_system(update_tile_color_on_player_move.system())
        .with_system(update_damaged_characters.system())
        .with_system(update_damaged_items.system())
        .with_system(update_camera.system())
        .with_system(update_log.system())
        .with_system(update_status.system())
}

trait AppBuilderExt {
    fn add_startup_system_set_to_stage(
        &mut self,
        startup_stage: StartupStage,
        system_set: SystemSet,
    ) -> &mut Self;
}

impl AppBuilderExt for AppBuilder {
    fn add_startup_system_set_to_stage(
        &mut self,
        startup_stage: StartupStage,
        system_set: SystemSet,
    ) -> &mut Self {
        self.app
            .schedule
            .stage(CoreStage::Startup, |schedule: &mut Schedule| {
                schedule.add_system_set_to_stage(startup_stage, system_set)
            });
        self
    }
}
