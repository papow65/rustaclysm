use crate::application::systems::maximize_window;
use crate::application::{check::check_delay, ApplicationState};
use crate::{background::BackgroundPlugin, gameplay::GameplayPlugin, hud::HudPlugin};
use crate::{keyboard::KeyboardPlugin, loading::LoadingIndicatorPlugin};
use crate::{main_menu::MainMenuPlugin, manual::ManualPlugin};
use crate::{pre_gameplay::PreGameplayPlugin, util::log_transition_plugin};
use bevy::prelude::{
    App, AppExit, AppExtStates, AssetPlugin, DefaultPlugins, Fixed, IVec2, ImagePlugin, Last,
    PluginGroup, Startup, Time, Window, WindowPlugin, WindowPosition,
};
use bevy::window::{PresentMode, WindowResolution};
use std::time::Duration;

pub(crate) fn run_application() -> AppExit {
    let mut app = App::new();

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
        PreGameplayPlugin,
        log_transition_plugin::<ApplicationState>,
    ));

    app.init_state::<ApplicationState>();
    app.enable_state_scoped_entities::<ApplicationState>();

    app.add_systems(Startup, maximize_window);
    app.add_systems(Last, check_delay);

    app.run()
}
