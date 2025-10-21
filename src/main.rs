mod check;
mod input;
mod log;
mod output;

use crate::{check::check_delay, input::create_global_key_bindings};
use crate::{log::log_archetypes, output::create_camera};
use application_state::ApplicationStatePlugin;
use background::BackgroundPlugin;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig};
use bevy::log::{DEFAULT_FILTER, Level, LogPlugin};
use bevy::prelude::{
    App, AppExit, AssetPlugin, DefaultPlugins, Fixed, FixedUpdate, ImagePlugin, Last,
    PluginGroup as _, Startup, Time, Window, WindowPlugin, info,
};
use bevy::window::PresentMode;
use gameplay::GameplayPlugin;
use git_version::git_version;
use hud::HudPlugin;
use keyboard::KeyboardPlugin;
use loading::LoadingIndicatorPlugin;
use main_menu::MainMenuPlugin;
use manual::ManualPlugin;
use pre_gameplay::PreGameplayPlugin;
use std::{env, fmt::Write as _, time::Duration};

fn main() -> AppExit {
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
                filter: log_filter(),
                ..LogPlugin::default()
            })
            .set(WindowPlugin {
                primary_window: Some(window),
                ..WindowPlugin::default()
            }),
    );

    if env::var("FPS_OVERLAY") == Ok(String::from("1")) {
        app.add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                frame_time_graph_config: FrameTimeGraphConfig::target_fps(120.0),
                ..FpsOverlayConfig::default()
            },
        });
    }

    // Now that the log plugin is created, we can log
    info!("Started Rustaclysm, version {}", git_version!());

    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(250)));

    app.add_plugins((
        ApplicationStatePlugin,
        HudPlugin,
        KeyboardPlugin,
        MainMenuPlugin,
        ManualPlugin,
        BackgroundPlugin,
        GameplayPlugin,
        LoadingIndicatorPlugin,
        PreGameplayPlugin,
    ));

    app.add_systems(Startup, (create_global_key_bindings, create_camera));
    app.add_systems(FixedUpdate, log_archetypes);
    app.add_systems(Last, check_delay);

    app.run()
}

fn log_filter() -> String {
    [
        "rustaclysm",
        "application_state",
        "background",
        "cdda_json_files",
        "gameplay",
        "gameplay_transition_state",
        "hud",
        "keyboard",
        "loading",
        "main_menu",
        "manual",
        "pre_gameplay",
        "units",
        "util",
    ]
    .into_iter()
    .fold(format!("info,{DEFAULT_FILTER}"), |mut acc, package| {
        write!(acc, ",{package}=debug").expect("Writing should work");
        acc
    })
}
