use crate::prelude::*;
use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

const SPACING: f32 = 20.0;
const FONT_SIZE: f32 = 40.0;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button = ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(250.0), Val::Px(70.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    };
    let font = default_font(&asset_server);

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                gap: Size::height(Val::Px(SPACING)),
                ..default()
            },
            ..default()
        })
        .insert(MenuRoot)
        .with_children(|parent| {
            parent
                .spawn(button.clone())
                .insert(ReturnButton)
                .with_children(|parent| add_text(parent, &font, "Return", GOOD_TEXT_COLOR));
            parent
                .spawn(button.clone())
                .insert(MainMenuButton)
                .with_children(|parent| add_text(parent, &font, "Main Menu", DEFAULT_TEXT_COLOR));
            parent
                .spawn(button)
                .insert(QuitButton)
                .with_children(|parent| add_text(parent, &font, "Quit", BAD_TEXT_COLOR));
        });
}

fn add_text(parent: &mut ChildBuilder, font: &Handle<Font>, text: &str, color: Color) {
    parent.spawn(TextBundle::from_section(
        text,
        TextStyle {
            font: font.clone(),
            font_size: FONT_SIZE,
            color,
        },
    ));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_menu_button_input(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    interaction_query: Query<
        (
            &Interaction,
            Option<&ReturnButton>,
            Option<&MainMenuButton>,
            Option<&QuitButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, return_button, main_menu_button, quit_button) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match (
                return_button.is_some(),
                main_menu_button.is_some(),
                quit_button.is_some(),
            ) {
                (true, false, false) => {
                    next_gameplay_state.set(GameplayScreenState::Base);
                }
                (false, true, false) => {
                    next_application_state.set(ApplicationState::MainMenu);
                }
                (false, false, true) => app_exit_events.send(AppExit),
                (return_button, main_menu_button, quit_button) => {
                    panic!("{return_button:?} {main_menu_button:?} {quit_button:?}");
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_menu_keyboard_input(
    mut key_events: EventReader<KeyboardInput>,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
) {
    for key_event in key_events.iter() {
        if key_event.state != ButtonState::Pressed {
            continue;
        }

        match key_event.key_code {
            Some(KeyCode::Escape) => next_gameplay_state.set(GameplayScreenState::Base),
            Some(KeyCode::M) => next_application_state.set(ApplicationState::MainMenu),
            Some(KeyCode::C | KeyCode::D | KeyCode::Q) => app_exit_events.send(AppExit),
            _ => {}
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_menu(mut commands: Commands, root_entities: Query<Entity, With<MenuRoot>>) {
    if let Ok(root_entity) = root_entities.get_single() {
        commands.entity(root_entity).despawn_recursive();
    }
}
