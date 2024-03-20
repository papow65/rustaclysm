use super::{
    check::check_delay,
    systems::{
        load_fonts, manage_button_hover, manage_global_keyboard_input, manage_scrolling,
        maximize_window,
    },
};
use crate::prelude::*;
use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseWheel},
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use std::time::Duration;

pub(crate) fn run_application() {
    let mut app = App::new();

    app.insert_resource(Msaa::default());

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: String::from('.'), // We add 'assets/' ourselves.
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

    app.insert_state(ApplicationState::MainMenu)
        .insert_state(ProgressScreenState::Complete);

    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(250)));

    app.add_plugins((
        MainMenuPlugin,
        CddaPlugin,
        GameplayPlugin,
        LoadingIndicatorPlugin,
    ));

    app.add_systems(Startup, (maximize_window, load_fonts));
    app.add_systems(
        Update,
        (
            manage_button_hover,
            manage_scrolling.run_if(on_event::<MouseWheel>()),
            manage_global_keyboard_input.run_if(on_event::<KeyboardInput>()),
        ),
    );
    app.add_systems(Last, check_delay);

    app.run();
}
