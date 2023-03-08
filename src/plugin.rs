use crate::prelude::*;
use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum UpdateSet {
    Behavior,
    Sync,
}

pub(crate) struct RustaclysmPlugin;

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
        app.add_startup_systems(
            (
                create_secondairy_resources,
                apply_system_buffers,
                spawn_initial_entities,
                spawn_hud,
                apply_system_buffers,
                maximize_window,
            )
                .chain(),
        );

        app.add_system(manage_mouse_input.before(update_camera));

        // executed every frame

        app.add_systems(
            (
                manage_keyboard_input,
                manage_characters,
                // Effect systems TODO in parallel
                manage_game_over,
                toggle_doors,
                update_damaged_characters,
                update_damaged_items,
                //
                apply_system_buffers,
            )
                .chain()
                .in_set(UpdateSet::Behavior),
        );
        app.add_systems(
            (
                // Updates TODO in parallel
                update_transforms,
                update_hidden_item_visibility,
                update_cursor_visibility_on_player_change,
                update_visualization_on_item_move,
                update_visualization_on_focus_move,
                update_camera,
            )
                .in_set(UpdateSet::Sync)
                .after(UpdateSet::Behavior),
        );

        app.add_system(update_log);
        app.add_system(update_status_fps);
        app.add_system(update_status_time);
        app.add_system(update_status_health);
        app.add_system(update_status_speed);
        app.add_system(update_status_player_state);
        app.add_system(update_status_detais);

        app.add_system(spawn_zones_for_camera.after(update_camera));
        app.add_system(update_collapsed_zone_levels.after(update_camera));

        app.add_system(check_delay.in_base_set(CoreSet::Last));

        /*bevy_mod_debugdump::print_main_schedule(app);
        std::process::exit(0);*/
    }
}
