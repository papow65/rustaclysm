use crate::application::systems::{
    enter_main_menu, manage_global_keyboard_input, maximize_window, preprocess_keyboard_input,
};
use crate::application::{check::check_delay, ApplicationState};
use crate::background::BackgroundPlugin;
use crate::common::{log_transition_plugin, Keys};
use crate::{gameplay::GameplayPlugin, hud::HudPlugin, loading::LoadingIndicatorPlugin};
use crate::{main_menu::MainMenuPlugin, manual::ManualPlugin};
use bevy::input::{keyboard::KeyboardInput, InputSystem};
use bevy::prelude::{
    on_event, App, AppExtStates, AssetPlugin, DefaultPlugins, Fixed, IVec2, ImagePlugin,
    IntoSystemConfigs, Last, Msaa, PluginGroup, PreUpdate, Startup, Time, Update, Window,
    WindowPlugin, WindowPosition,
};
use bevy::window::{PresentMode, WindowResolution};
use std::time::Duration;

pub(crate) fn run_application() {
    let mut app = App::new();

    app.insert_resource(Msaa::default())
        .insert_resource(Keys::default());

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
    app.add_systems(PreUpdate, preprocess_keyboard_input.after(InputSystem));
    app.add_systems(
        Update,
        manage_global_keyboard_input.run_if(on_event::<KeyboardInput>()),
    );
    app.add_systems(Last, check_delay);

    app.run();
}
