use crate::prelude::*;
use bevy::{input::ButtonState, prelude::*, utils::HashMap};

const SPACING: f32 = 5.0;
const FONT_SIZE: f32 = 16.0;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_inventory(mut commands: Commands, fonts: Res<Fonts>) {
    let panel = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
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
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ..default()
        })
        .insert(StateBound::<GameplayScreenState>::default())
        .add_child(panel);

    commands.insert_resource(InventoryScreen {
        panel,
        selected_item: None,
        previous_items: HashMap::default(),
        next_items: HashMap::default(),
        drop_direction: HorizontalDirection::Here,
        section_text_style: TextStyle {
            font: fonts.default(),
            font_size: FONT_SIZE,
            color: SOFT_TEXT_COLOR,
        },
        selected_section_text_style: TextStyle {
            font: fonts.default(),
            font_size: FONT_SIZE,
            color: WARN_TEXT_COLOR,
        },
        item_text_style: TextStyle {
            font: fonts.default(),
            font_size: FONT_SIZE,
            color: DEFAULT_TEXT_COLOR,
        },
        selected_item_text_style: TextStyle {
            font: fonts.default(),
            font_size: FONT_SIZE,
            color: GOOD_TEXT_COLOR,
        },
        last_time: Timestamp::ZERO,
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn clear_inventory(
    clock: Clock,
    mut inventory: ResMut<InventoryScreen>,
    children: Query<&Children>,
    mut styles: Query<&mut Style>,
) -> bool {
    if inventory.last_time == clock.time() {
        return false;
    }
    inventory.last_time = clock.time();

    if let Ok(children) = children.get(inventory.panel) {
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
    mut inventory: ResMut<InventoryScreen>,
    players: Query<(&Pos, &BodyContainers), With<Player>>,
    items: Query<(Item, Option<&Corpse>, Option<&Integrity>, &LastSeen)>,
) {
    if !run.0 {
        return;
    }

    inventory.previous_items.clear();
    inventory.next_items.clear();

    let (&player_pos, body_containers) = players.single();
    let items_by_section = items_by_section(&envir, &items, player_pos, body_containers);
    let mut items_by_section = items_by_section.into_iter().collect::<Vec<_>>();
    items_by_section.sort_by_key(|(section, _)| (*section).clone());

    let selected_item_present = items_by_section.iter().any(|(_, items)| {
        items
            .iter()
            .any(|(e, _, _)| Some(*e) == inventory.selected_item)
    });

    commands.entity(inventory.panel).with_children(|parent| {
        let mut first_item = None;
        let mut previous_item = None;

        for (section, items) in items_by_section {
            let section_style = if section == InventorySection::Nbor(inventory.drop_direction) {
                &inventory.selected_section_text_style
            } else {
                &inventory.section_text_style
            };

            let mut text_sections = vec![TextSection::new(
                format!("[{section}]"),
                section_style.clone(),
            )];
            if section == InventorySection::Nbor(inventory.drop_direction) {
                text_sections.push(TextSection::new(
                    String::from("(dropping here)"),
                    section_style.clone(),
                ));
            }
            parent.spawn(TextBundle::from_sections(text_sections));

            for (entity, id, item_phrase) in items {
                if first_item.is_none() {
                    first_item = Some(entity);
                    if !selected_item_present {
                        inventory.selected_item = Some(entity);
                    }
                }

                if let Some(item_info) = infos.item(id) {
                    let item_style = if Some(entity) == inventory.selected_item {
                        &inventory.selected_item_text_style
                    } else {
                        &inventory.item_text_style
                    };

                    add_row(
                        &section,
                        parent,
                        entity,
                        &item_phrase,
                        item_info,
                        item_style,
                    );
                }
                if let Some(previous_item) = previous_item {
                    inventory.previous_items.insert(entity, previous_item);
                    inventory.next_items.insert(previous_item, entity);
                }
                previous_item = Some(entity);
            }

            // empty line
            parent.spawn(TextBundle::from_section(
                String::from(" "),
                inventory.section_text_style.clone(),
            ));
        }

        assert_eq!(
            first_item.is_some(),
            previous_item.is_some(),
            "If there is a first item, there also should be a last item."
        );
        if let Some(previous_item) = previous_item {
            inventory
                .previous_items
                .insert(first_item.unwrap(), previous_item);
            inventory
                .next_items
                .insert(previous_item, first_item.unwrap());
        }
    });
}

fn items_by_section<'a>(
    envir: &'a Envir,
    items: &'a Query<(Item, Option<&Corpse>, Option<&Integrity>, &LastSeen)>,
    player_pos: Pos,
    body_containers: &'a BodyContainers,
) -> HashMap<InventorySection, Vec<(Entity, &'a ObjectId, Phrase)>> {
    let mut fields_by_section = HashMap::default();
    for (nbor, nbor_pos) in envir.nbors_for_item_handling(player_pos) {
        fields_by_section.insert(
            InventorySection::Nbor(nbor.horizontal_projection()),
            items
                .iter()
                .filter(|(item, .., last_seen)| {
                    *last_seen == &LastSeen::Currently && Some(&nbor_pos) == item.pos
                })
                .collect::<Vec<_>>(),
        );
    }
    fields_by_section.insert(
        InventorySection::Hands,
        items
            .iter()
            .filter(|(item, ..)| item.parent.get() == body_containers.hands)
            .collect::<Vec<_>>(),
    );
    fields_by_section.insert(
        InventorySection::Clothing,
        items
            .iter()
            .filter(|(item, ..)| item.parent.get() == body_containers.clothing)
            .collect::<Vec<_>>(),
    );

    let mut items_by_section = HashMap::default();
    for (section, fields_iter) in fields_by_section {
        let mut items = fields_iter
            .into_iter()
            .map(|(item, ..)| {
                (
                    item.entity,
                    &item.definition.id,
                    Phrase::from_fragments(item.fragments()),
                )
            })
            .collect::<Vec<_>>();
        items.sort_by_key(|(.., phrase)| format!("{phrase}"));
        items_by_section.insert(section, items);
    }
    items_by_section
}

fn add_row(
    section: &InventorySection,
    parent: &mut ChildBuilder,
    entity: Entity,
    item_phrase: &Phrase,
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
                        item_phrase.as_text_sections(item_syle),
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
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut inventory: ResMut<InventoryScreen>,
) {
    for (state, combo) in keys.combos() {
        if state != ButtonState::Pressed {
            continue;
        }

        match combo {
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Escape) | KeyCombo::Character('i') => {
                next_gameplay_state.set(GameplayScreenState::Base);
            }
            KeyCombo::KeyCode(
                Ctrl::Without,
                KeyCode::Numpad1
                | KeyCode::Numpad2
                | KeyCode::Numpad3
                | KeyCode::Numpad4
                | KeyCode::Numpad5
                | KeyCode::Numpad6
                | KeyCode::Numpad7
                | KeyCode::Numpad8
                | KeyCode::Numpad9,
            ) => {
                drop_at(&mut inventory, &combo);
            }
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Up) => {
                select_up(&mut inventory);
            }
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Down) => {
                select_down(&mut inventory);
            }
            KeyCombo::KeyCode(
                Ctrl::Without,
                key_code @ (KeyCode::D | KeyCode::T | KeyCode::U | KeyCode::W),
            ) => {
                handle_selected_item(&mut inventory, &mut instruction_queue, key_code);
            }
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::E) => {
                // Special case, because we don't want to select another item after the action.
                examine_selected_item(&inventory, &mut instruction_queue);
            }
            _ => {}
        }
    }
}

fn drop_at(inventory: &mut InventoryScreen, combo: &KeyCombo) {
    let nbor = PlayerDirection::try_from(combo)
        .expect("The direction should be valid ({combo:?})")
        .to_nbor();
    match nbor {
        Nbor::Horizontal(horizontal_direction) => {
            inventory.drop_direction = horizontal_direction;
            inventory.last_time = Timestamp::ZERO;
        }
        _ => panic!("Only horizontal dropping allowed"),
    }
}

fn select_up(inventory: &mut InventoryScreen) {
    if let Some(selected) = inventory.selected_item {
        inventory.selected_item = inventory.previous_items.get(&selected).copied();
        inventory.last_time = Timestamp::ZERO;
    }
}

fn select_down(inventory: &mut InventoryScreen) {
    if let Some(selected) = inventory.selected_item {
        inventory.selected_item = inventory.next_items.get(&selected).copied();
        inventory.last_time = Timestamp::ZERO;
    }
}

fn handle_selected_item(
    inventory: &mut ResMut<InventoryScreen>,
    instruction_queue: &mut ResMut<InstructionQueue>,
    key_code: KeyCode,
) {
    if let Some(selected_item) = inventory.selected_item {
        let next_item = inventory.next_items.get(&selected_item).copied();
        instruction_queue.add(match key_code {
            KeyCode::D => QueuedInstruction::Dump(selected_item, inventory.drop_direction),
            KeyCode::T => QueuedInstruction::Pickup(selected_item),
            KeyCode::U => QueuedInstruction::Unwield(selected_item),
            KeyCode::W => QueuedInstruction::Wield(selected_item),
            _ => panic!("Unexpected key {key_code:?}"),
        });
        inventory.selected_item = next_item;
    }
}

fn examine_selected_item(
    inventory: &ResMut<InventoryScreen>,
    instruction_queue: &mut ResMut<InstructionQueue>,
) {
    if let Some(selected_item) = inventory.selected_item {
        instruction_queue.add(QueuedInstruction::ExamineItem(selected_item));
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_inventory_button_input(
    mut instruction_queue: ResMut<InstructionQueue>,
    interactions: Query<(&Interaction, &ActionButton), (Changed<Interaction>, With<Button>)>,
    inventory: Res<InventoryScreen>,
) {
    for (&interaction, &ActionButton(entity, ref inventory_action)) in interactions.iter() {
        if interaction == Interaction::Pressed {
            println!("{inventory_action} {entity:?}");
            let instruction = match inventory_action {
                InventoryAction::Examine => QueuedInstruction::ExamineItem(entity),
                InventoryAction::Take => QueuedInstruction::Pickup(entity),
                InventoryAction::Drop => QueuedInstruction::Dump(entity, inventory.drop_direction),
                InventoryAction::Wield => QueuedInstruction::Wield(entity),
                InventoryAction::Unwield => QueuedInstruction::Unwield(entity),
            };
            instruction_queue.add(instruction);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn remove_inventory_resource(mut commands: Commands) {
    commands.remove_resource::<InventoryScreen>();
}
