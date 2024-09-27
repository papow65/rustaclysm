use crate::application::systems::{enter_main_menu, maximize_window};
use crate::application::{check::check_delay, ApplicationState};
use crate::background::BackgroundPlugin;
use crate::common::log_transition_plugin;
use crate::{gameplay::GameplayPlugin, hud::HudPlugin, keyboard::KeyboardPlugin};
use crate::{loading::LoadingIndicatorPlugin, main_menu::MainMenuPlugin, manual::ManualPlugin};
use bevy::prelude::{
    App, AppExtStates, AssetPlugin, DefaultPlugins, Fixed, IVec2, ImagePlugin, Last, Msaa,
    PluginGroup, Startup, Time, Window, WindowPlugin, WindowPosition,
};
use bevy::window::{PresentMode, WindowResolution};
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

    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(250)));

    app.add_plugins((
        HudPlugin,
        KeyboardPlugin,
        MainMenuPlugin,
        ManualPlugin,
        BackgroundPlugin,
        GameplayPlugin,
        LoadingIndicatorPlugin,
        log_transition_plugin::<ApplicationState>,
    ));

    app.insert_state(ApplicationState::Startup);
    app.enable_state_scoped_entities::<ApplicationState>();

    app.add_systems(Startup, (maximize_window, enter_main_menu));
    app.add_systems(Last, check_delay);

    app.run();
}
