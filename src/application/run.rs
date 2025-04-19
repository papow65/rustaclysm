use crate::application::{check::check_delay, input::create_global_key_bindings};
use crate::{background::BackgroundPlugin, main_menu::MainMenuPlugin};
use crate::{loading::LoadingIndicatorPlugin, pre_gameplay::PreGameplayPlugin};
use application_state::ApplicationState;
use bevy::log::{DEFAULT_FILTER, Level, LogPlugin};
use bevy::prelude::{
    App, AppExit, AppExtStates as _, AssetPlugin, DefaultPlugins, Fixed, ImagePlugin, Last,
    PluginGroup as _, Startup, Time, Window, WindowPlugin, info,
};
use bevy::window::PresentMode;
use gameplay::GameplayPlugin;
use git_version::git_version;
use hud::HudPlugin;
use keyboard::KeyboardPlugin;
use manual::ManualPlugin;
use std::time::Duration;
use util::log_transition_plugin;

pub(crate) fn run_application() -> AppExit {
    let mut app = App::new();

    let mut window = Window {
        title: String::from("Rustaclysm"),
        present_mode: PresentMode::Mailbox, // much better responsiveness
        ..Window::default()
    };
    window.set_maximized(true);

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: String::from('.'), // We add 'assets/' ourselves.
                ..AssetPlugin::default()
            })
            .set(ImagePlugin::default_nearest())
            .set(LogPlugin {
                level: Level::DEBUG,
                filter: String::from("info,rustaclysm=debug,cdda_json_files=debug,units=debug,")
                    + DEFAULT_FILTER,
                ..LogPlugin::default()
            })
            .set(WindowPlugin {
                primary_window: Some(window),
                ..WindowPlugin::default()
            }),
    );

    // Now that the log plugin is created, we can log
    info!("Started Rustaclysm, version {}", git_version!());

    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(250)));

    app.add_plugins((
        HudPlugin,
        KeyboardPlugin,
        MainMenuPlugin,
        ManualPlugin,
        BackgroundPlugin,
        GameplayPlugin,
        LoadingIndicatorPlugin,
        PreGameplayPlugin,
        log_transition_plugin::<ApplicationState>,
    ));

    app.init_state::<ApplicationState>();
    app.enable_state_scoped_entities::<ApplicationState>();

    app.add_systems(Startup, create_global_key_bindings);

    app.add_systems(Last, check_delay);

    app.run()
}
