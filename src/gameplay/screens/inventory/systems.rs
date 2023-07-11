use crate::prelude::*;
use bevy::{app::AppExit, input::ButtonState, prelude::*, utils::HashMap};

const SPACING: f32 = 5.0;
const FONT_SIZE: f32 = 16.0;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_inventory(mut commands: Commands, fonts: Res<Fonts>) {
    let root = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
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
        .id();

    commands.insert_resource(InventoryScreen {
        root,
        section_text_style: TextStyle {
            font: fonts.default(),
            font_size: FONT_SIZE,
            color: DEFAULT_TEXT_COLOR,
        },
        item_text_style: TextStyle {
            font: fonts.default(),
            font_size: FONT_SIZE,
            color: DEFAULT_TEXT_COLOR,
        },
        drop_direction: HorizontalDirection::Here,
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn clear_inventory(
    clock: Clock,
    inventory: Res<InventoryScreen>,
    mut last_time: Local<Milliseconds>,
    children: Query<&Children>,
    mut styles: Query<&mut Style>,
) -> bool {
    if *last_time == clock.time() {
        return false;
    }
    *last_time = clock.time();

    if let Ok(children) = children.get(inventory.root) {
        for &child in children {
            if let Ok(mut style) = styles.get_mut(child) {
                style.display = Display::None;
            }
        }
    }

    true
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_inventory(
    run: In<bool>,
    mut commands: Commands,
    envir: Envir,
    infos: Res<Infos>,
    inventory: Res<InventoryScreen>,
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
    if !run.0 {
        return;
    }

    let (&player_pos, body_containers) = players.single();
    let items_by_section = items_by_section(&envir, &items, player_pos, body_containers);

    let mut items_by_section = items_by_section.into_iter().collect::<Vec<_>>();
    items_by_section.sort_by_key(|(section, _)| (*section).clone());

    commands.entity(inventory.root).with_children(|parent| {
        for (section, items) in items_by_section {
            parent.spawn(TextBundle::from_section(
                format!("[{section}]"),
                inventory.section_text_style.clone(),
            ));

            if section == InventorySection::Nbor(inventory.drop_direction) {
                parent.spawn(TextBundle::from_section(
                    String::from("(dropping here)"),
                    inventory.section_text_style.clone(),
                ));
            }

            for (entity, id, item_message) in items {
                if let Some(item_info) = infos.item(id) {
                    add_row(
                        &section,
                        parent,
                        entity,
                        item_message,
                        item_info,
                        &inventory.item_text_style,
                    );
                }
            }

            // empty line
            parent.spawn(TextBundle::from_section(
                String::from(" "),
                inventory.section_text_style.clone(),
            ));
        }
    });
}

fn items_by_section<'a>(
    envir: &'a Envir,
    items: &'a Query<(
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
    player_pos: Pos,
    body_containers: &'a BodyContainers,
) -> HashMap<InventorySection, Vec<(Entity, &'a ObjectId, Message)>> {
    let mut fields_by_section = HashMap::default();
    for (nbor, nbor_pos) in envir.nbors_for_item_handling(player_pos) {
        fields_by_section.insert(
            InventorySection::Nbor(nbor.horizontal_projection()),
            items
                .iter()
                .filter(|(_, _, pos, .., last_seen)| {
                    last_seen == &&LastSeen::Currently && &Some(&nbor_pos) == pos
                })
                .collect::<Vec<_>>(),
        );
    }
    fields_by_section.insert(
        InventorySection::Hands,
        items
            .iter()
            .filter(|(_, parent, ..)| parent.map_or(false, |p| p.get() == body_containers.hands))
            .collect::<Vec<_>>(),
    );
    fields_by_section.insert(
        InventorySection::Clothing,
        items
            .iter()
            .filter(|(_, parent, ..)| parent.map_or(false, |p| p.get() == body_containers.clothing))
            .collect::<Vec<_>>(),
    );

    let mut items_by_section = HashMap::default();
    for (section, fields_iter) in fields_by_section {
        let mut items = fields_iter
            .into_iter()
            .map(
                |(entity, _parent, _pos, definition, name, amount, filthy, ..)| {
                    (
                        entity,
                        &definition.id,
                        Message::info().extend(name.as_item(Some(amount), filthy)),
                    )
                },
            )
            .collect::<Vec<_>>();
        items.sort_by_key(|(.., message)| format!("{}", &message));
        items_by_section.insert(section, items);
    }
    items_by_section
}

fn add_row(
    section: &InventorySection,
    parent: &mut ChildBuilder,
    entity: Entity,
    item_message: Message,
    item_info: &ItemInfo,
    item_syle: &TextStyle,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                column_gap: Val::Px(SPACING),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        overflow: Overflow::clip(),
                        ..Style::default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_sections(
                        item_message.into_text_sections(item_syle),
                    ));
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        overflow: Overflow::clip(),
                        justify_content: JustifyContent::End,
                        ..Style::default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        if let Some(ref volume) = item_info.volume {
                            format!("{volume}")
                        } else {
                            String::new()
                        },
                        item_syle.clone(),
                    ));
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        overflow: Overflow::clip(),
                        justify_content: JustifyContent::End,
                        ..Style::default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        if let Some(ref mass) = item_info.mass {
                            format!("{mass}")
                        } else {
                            String::new()
                        },
                        item_syle.clone(),
                    ));
                });

            let mut actions = vec![InventoryAction::Examine];
            if matches!(section, InventorySection::Nbor(_)) {
                actions.push(InventoryAction::Take);
            } else {
                actions.push(InventoryAction::Drop);
            }
            if matches!(section, InventorySection::Hands) {
                actions.push(InventoryAction::Unwield);
            } else {
                actions.push(InventoryAction::Wield);
            }

            for action in actions {
                parent
                    .spawn(ButtonBundle {
                        style: Style {
                            width: Val::Px(70.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: DEFAULT_BUTTON_COLOR.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            format!("{}", &action),
                            item_syle.clone(),
                        ));
                    })
                    .insert(ActionButton(entity, action));
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
pub(crate) fn manage_inventory_button_input(
    mut instruction_queue: ResMut<InstructionQueue>,
    interactions: Query<(&Interaction, &ActionButton), (Changed<Interaction>, With<Button>)>,
) {
    for (&interaction, &ActionButton(entity, ref inventory_action)) in interactions.iter() {
        if interaction == Interaction::Pressed {
            println!("{inventory_action} {entity:?}");
            let instruction = match inventory_action {
                InventoryAction::Examine => QueuedInstruction::ExamineItem(entity),
                InventoryAction::Take => QueuedInstruction::Pickup(entity),
                InventoryAction::Drop => QueuedInstruction::Dump(entity),
                InventoryAction::Wield => QueuedInstruction::Wield(entity),
                InventoryAction::Unwield => QueuedInstruction::Unwield(entity),
            };
            instruction_queue.add(instruction);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_inventory(mut commands: Commands, inventory: Res<InventoryScreen>) {
    commands.entity(inventory.root).despawn_recursive();
    commands.remove_resource::<InventoryScreen>();
}
