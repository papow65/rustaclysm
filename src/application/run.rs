use crate::application::systems::{
    enter_main_menu, load_fonts, manage_button_color, manage_global_keyboard_input,
    manage_scrolling_lists, maximize_window, preprocess_keyboard_input, resize_scrolling_lists,
};
use crate::application::{check::check_delay, ApplicationState};
use crate::prelude::{log_transition_plugin, CddaPlugin, GameplayPlugin, Keys};
use crate::{loading::LoadingIndicatorPlugin, main_menu::MainMenuPlugin};
use bevy::input::{keyboard::KeyboardInput, mouse::MouseWheel, InputSystem};
use bevy::prelude::{
    on_event, resource_exists_and_changed, App, AppExtStates, AssetPlugin, Condition,
    DefaultPlugins, Fixed, IVec2, ImagePlugin, IntoSystemConfigs, Last, Msaa, PluginGroup,
    PreUpdate, Startup, Time, UiScale, Update, Window, WindowPlugin, WindowPosition,
};
use bevy::window::{PresentMode, WindowResized, WindowResolution};
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
        MainMenuPlugin,
        CddaPlugin,
        GameplayPlugin,
        LoadingIndicatorPlugin,
        log_transition_plugin::<ApplicationState>,
    ));

    app.insert_state(ApplicationState::Startup);
    app.enable_state_scoped_entities::<ApplicationState>();

    app.add_systems(
        Startup,
        (maximize_window, (load_fonts, enter_main_menu).chain()),
    );
    app.add_systems(PreUpdate, preprocess_keyboard_input.after(InputSystem));
    app.add_systems(
        Update,
        (
            manage_button_color,
            manage_scrolling_lists.run_if(on_event::<MouseWheel>()),
            manage_global_keyboard_input.run_if(on_event::<KeyboardInput>()),
            resize_scrolling_lists.run_if(
                on_event::<WindowResized>().or_else(resource_exists_and_changed::<UiScale>),
            ),
        ),
    );
    app.add_systems(Last, check_delay);

    app.run();
}
