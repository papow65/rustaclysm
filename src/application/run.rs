use crate::prelude::*;
use bevy::{
    prelude::*,
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
    );

    app.add_state::<ApplicationState>();

    app.add_plugin(MainMenuPlugin)
        .add_plugin(CddaPlugin)
        .add_plugin(GameplayPlugin);

    // once at startup
    app.add_startup_system(maximize_window);

    // every frame
    app.add_system(manage_button_hover);

    //bevy_mod_debugdump::print_main_schedule(&mut app);

    app.run();
}
