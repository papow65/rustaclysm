use crate::prelude::{
    maximize_window, ApplicationState, CddaPlugin, GameplayPlugin, MainMenuPlugin,
};
use bevy::{
    prelude::{
        App, AssetPlugin, DefaultPlugins, IVec2, ImagePlugin, Msaa, PluginGroup, Window,
        WindowPlugin, WindowPosition,
    },
    window::{PresentMode, WindowResolution},
};

pub(crate) fn run_application() {
    let mut app = App::new();

    app.insert_resource(Msaa::default());

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                asset_folder: String::from('.'), // We add 'assets/' ourselves.
                ..AssetPlugin::default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Rustaclysm"),
                    present_mode: PresentMode::Mailbox, // much better responsiveness
                    resolution: WindowResolution::new(50.0, 40.0),
                    position: WindowPosition::At(IVec2::new(10, 10)),
                    ..Window::default()
                }),
                ..WindowPlugin::default()
            }),
        /*.edit_schedule(bevy::prelude::CoreSchedule::Main, |schedule| {
                 *            chedule.set_build_settings(bevy::ecs::*schedule::ScheduleBuildSettings {
                 *                ambiguity_detection: bevy::ecs::schedule::LogLevel::Warn,
                 *                ..bevy::ecs::schedule::ScheduleBuildSettings::default()
        });
        })*/
    );

    app.add_state::<ApplicationState>();

    app.add_plugin(MainMenuPlugin)
        .add_plugin(CddaPlugin)
        .add_plugin(GameplayPlugin);

    app.add_startup_system(maximize_window);

    //bevy_mod_debugdump::print_main_schedule(app);

    app.run();
}
