use crate::prelude::*;
use bevy::{app::AppExit, input::ButtonState, prelude::*, utils::HashMap};

const SPACING: f32 = 20.0;
const FONT_SIZE: f32 = 16.0;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_inventory(
    mut commands: Commands,
    fonts: Res<Fonts>,
    envir: Envir,
    players: Query<(&Pos, &BodyContainers), With<Player>>,
    items: Query<(
        Entity,
        Option<&Parent>,
        Option<&Pos>,
        &ObjectDefinition,
        &ObjectName,
        &Amount,
        Option<&Filthy>,
        Option<&Corpse>,
        Option<&Integrity>,
        &LastSeen,
    )>,
) {
    let item_text_syle = TextStyle {
        font: fonts.default(),
        font_size: FONT_SIZE,
        color: DEFAULT_TEXT_COLOR,
    };
    let location_text_syle = TextStyle {
        color: WARN_TEXT_COLOR,
        ..item_text_syle.clone()
    };

    let (&player_pos, body_containers) = players.single();

    let mut fields_by_section = HashMap::default();
    for (nbor, nbor_pos) in envir.nbors_for_item_handling(player_pos) {
        fields_by_section.insert(
            InventorySection::Nbor(nbor),
            items
                .iter()
                .filter(|(_, _, pos, _, _, _, _, _, _, last_seen)| {
                    last_seen == &&LastSeen::Currently && &Some(&nbor_pos) == pos
                })
                .collect::<Vec<_>>(),
        );
    }
    fields_by_section.insert(
        InventorySection::Hands,
        items
            .iter()
            .filter(|(_, parent, _, _, _, _, _, _, _, _)| {
                parent.map_or(false, |p| p.get() == body_containers.hands)
            })
            .collect::<Vec<_>>(),
    );
    fields_by_section.insert(
        InventorySection::Clothing,
        items
            .iter()
            .filter(|(_, parent, _, _, _, _, _, _, _, _)| {
                parent.map_or(false, |p| p.get() == body_containers.clothing)
            })
            .collect::<Vec<_>>(),
    );

    let mut items_by_section = HashMap::default();
    for (section, fields_iter) in fields_by_section {
        let mut items = fields_iter
            .into_iter()
            .map(
                |(
                    _entity,
                    _parent,
                    _pos,
                    _definition,
                    name,
                    amount,
                    filthy,
                    _corpse,
                    _integrity,
                    _last_seen,
                )| { Message::info().extend(name.as_item(Some(amount), filthy)) },
            )
            .collect::<Vec<_>>();
        items.sort_by_key(|message| format!("{}", &message));
        items_by_section.insert(section, items);
    }

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                margin: UiRect::horizontal(Val::Px(360.0)),
                padding: UiRect::all(Val::Px(SPACING)),
                ..default()
            },
            background_color: PANEL_COLOR.into(),
            ..default()
        })
        .insert(InventoryRoot)
        .with_children(|parent| {
            let mut items_by_section = items_by_section.into_iter().collect::<Vec<_>>();
            items_by_section.sort_by_key(|(section, _)| (*section).clone());
            for (section, items) in items_by_section {
                if !items.is_empty() {
                    parent.spawn(TextBundle::from_section(
                        format!("[{section}]"),
                        location_text_syle.clone(),
                    ));

                    for item_message in items {
                        parent.spawn(TextBundle::from_sections(
                            item_message.into_text_sections(&item_text_syle),
                        ));
                    }
                }
            }
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_inventory_keyboard_input(
    mut keys: Keys,
    _next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
) {
    for (state, combo) in keys.combos() {
        if state != ButtonState::Pressed {
            continue;
        }

        match combo {
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Escape) | KeyCombo::Character('i') => {
                next_gameplay_state.set(GameplayScreenState::Base);
            }
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::C | KeyCode::Q) => {
                app_exit_events.send(AppExit);
            }
            _ => {}
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_inventory(
    mut commands: Commands,
    root_entities: Query<Entity, With<InventoryRoot>>,
) {
    if let Ok(root_entity) = root_entities.get_single() {
        commands.entity(root_entity).despawn_recursive();
    }
}
