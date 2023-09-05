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

    app.add_state::<ApplicationState>()
        .add_state::<ProgressScreenState>();

    app.insert_resource(FixedTime::new_from_secs(0.25));

    app.add_plugins((
        MainMenuPlugin,
        CddaPlugin,
        GameplayPlugin,
        LoadingIndicatorPlugin,
    ));

    app.add_systems(Startup, (maximize_window, load_fonts));
    app.add_systems(Update, (manage_button_hover, manage_global_keyboard_input));
    app.add_systems(Last, check_delay);

    app.run();
}
