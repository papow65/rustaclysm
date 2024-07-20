use super::{
    components::{InventoryAction, InventoryButton},
    resource::InventoryScreen,
    section::InventorySection,
};
use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_inventory(mut commands: Commands, fonts: Res<Fonts>) {
    let panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            },
            ScrollingList::default(),
        ))
        .id();
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
            StateScoped(GameplayScreenState::Inventory),
        ))
        .with_children(|builder| {
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Auto,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        margin: UiRect::px(10.0, 365.0, 10.0, 10.0),
                        padding: UiRect::all(SMALL_SPACING),
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    background_color: PANEL_COLOR.into(),
                    ..default()
                })
                .add_child(panel);
        });

    commands.insert_resource(InventoryScreen {
        panel,
        selection_list: SelectionList::default(),
        selected_row: None,
        drop_direction: HorizontalDirection::Here,
        section_text_style: fonts.regular(SOFT_TEXT_COLOR),
        selected_section_text_style: fonts.regular(WARN_TEXT_COLOR),
        item_text_style: fonts.regular(DEFAULT_TEXT_COLOR),
        selected_item_text_style: fonts.regular(GOOD_TEXT_COLOR),
        last_time: Timestamp::ZERO,
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn clear_inventory(
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
pub(super) fn update_inventory(
    In(run): In<bool>,
    mut commands: Commands,
    envir: Envir,
    infos: Res<Infos>,
    mut inventory: ResMut<InventoryScreen>,
    players: Query<(&Pos, &BodyContainers), With<Player>>,
    items: Query<(Item, Option<&Corpse>, Option<&Integrity>, &LastSeen)>,
) {
    if !run {
        return;
    }

    inventory.selection_list.clear(false);

    let (&player_pos, body_containers) = players.single();
    let items_by_section = items_by_section(&envir, &items, player_pos, body_containers);
    let mut items_by_section = items_by_section.into_iter().collect::<Vec<_>>();
    items_by_section.sort_by_key(|(section, _)| (*section).clone());

    commands.entity(inventory.panel).with_children(|parent| {
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
                inventory.selection_list.append(entity);

                if let Some(item_info) = infos.try_item(id) {
                    let item_style = if Some(entity) == inventory.selection_list.selected {
                        &inventory.selected_item_text_style
                    } else {
                        &inventory.item_text_style
                    };

                    let row_entity = add_row(
                        &section,
                        parent,
                        entity,
                        &item_phrase,
                        item_info,
                        item_style,
                    );

                    if Some(entity) == inventory.selection_list.selected {
                        inventory.selected_row = Some(row_entity);
                    }
                } else {
                    eprintln!("Unknown item: {id:?}");
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
) -> Entity {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                column_gap: SMALL_SPACING,
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
                    .spawn((ButtonBundle {
                        style: Style {
                            width: Val::Px(70.0),
                            justify_content: JustifyContent::Center,
                            ..Style::default()
                        },
                        ..ButtonBundle::default()
                    },))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            format!("{}", &action),
                            item_syle.clone(),
                        ));
                    })
                    .insert(InventoryButton(entity, action));
            }
        })
        .id()
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_inventory_keyboard_input(
    keys: Res<Keys>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut inventory: ResMut<InventoryScreen>,
    items: Query<(&Transform, &Node)>,
    mut scrolling_lists: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    scrolling_parents: Query<(&Node, &Style), Without<ScrollingList>>,
) {
    for key_change in keys.without_ctrl() {
        match key_change.key {
            Key::Code(KeyCode::Escape) | Key::Character('i')
                if key_change.change == InputChange::JustPressed =>
            {
                next_gameplay_state.set(GameplayScreenState::Base);
            }
            Key::Code(
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
                drop_at(&mut inventory, &key_change.key);
            }
            Key::Code(
                key_code @ (KeyCode::ArrowUp
                | KeyCode::ArrowDown
                | KeyCode::PageUp
                | KeyCode::PageDown),
            ) => {
                inventory.adjust_selection(&key_code);
                follow_selected(&inventory, &items, &mut scrolling_lists, &scrolling_parents);
            }
            Key::Character(char @ ('d' | 't' | 'u' | 'w')) => {
                handle_selected_item(&mut inventory, &mut instruction_queue, char);
            }
            Key::Character('e') => {
                // Special case, because we don't want to select another item after the action.
                examine_selected_item(&inventory, &mut instruction_queue);
            }
            _ => {}
        }
    }
}

fn drop_at(inventory: &mut InventoryScreen, key: &Key) {
    let nbor = PlayerDirection::try_from(key)
        .expect("The direction should be valid ({key:?})")
        .to_nbor();
    match nbor {
        Nbor::Horizontal(horizontal_direction) => {
            inventory.drop_direction = horizontal_direction;
            inventory.last_time = Timestamp::ZERO;
        }
        _ => panic!("Only horizontal dropping allowed"),
    }
}

fn follow_selected(
    inventory: &InventoryScreen,
    items: &Query<(&Transform, &Node)>,
    scrolling_lists: &mut Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    scrolling_parents: &Query<(&Node, &Style), Without<ScrollingList>>,
) {
    // inventory.selection_list.selected refers to the selected item entity
    let Some(selected_row) = inventory.selected_row else {
        return;
    };

    let (item_transform, item_node) = items
        .get(selected_row)
        .expect("Selected item should be found");

    let (mut scrolling_list, mut style, parent, list_node) = scrolling_lists.single_mut();
    let (parent_node, parent_style) = scrolling_parents
        .get(parent.get())
        .expect("Parent node should be found");
    style.top = scrolling_list.follow(
        item_transform,
        item_node,
        list_node,
        parent_node,
        parent_style,
    );
}

fn handle_selected_item(
    inventory: &mut InventoryScreen,
    instruction_queue: &mut InstructionQueue,
    char: char,
) {
    if let Some(selected_item) = inventory.selection_list.selected {
        inventory
            .selection_list
            .adjust(StepSize::Single, StepDirection::Down);
        instruction_queue.add(
            match char {
                'd' => QueuedInstruction::Dump(selected_item, inventory.drop_direction),
                't' => QueuedInstruction::Pickup(selected_item),
                'u' => QueuedInstruction::Unwield(selected_item),
                'w' => QueuedInstruction::Wield(selected_item),
                _ => panic!("Unexpected key {char:?}"),
            },
            InputChange::Held,
        );
    }
}

fn examine_selected_item(inventory: &InventoryScreen, instruction_queue: &mut InstructionQueue) {
    if let Some(selected_item) = inventory.selection_list.selected {
        instruction_queue.add(
            QueuedInstruction::ExamineItem(selected_item),
            InputChange::Held,
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_inventory_button_input(
    mut instruction_queue: ResMut<InstructionQueue>,
    interactions: Query<(&Interaction, &InventoryButton), (Changed<Interaction>, With<Button>)>,
    inventory: Res<InventoryScreen>,
) {
    for (&interaction, &InventoryButton(entity, ref inventory_action)) in interactions.iter() {
        if interaction == Interaction::Pressed {
            println!("{inventory_action} {entity:?}");
            let instruction = match inventory_action {
                InventoryAction::Examine => QueuedInstruction::ExamineItem(entity),
                InventoryAction::Take => QueuedInstruction::Pickup(entity),
                InventoryAction::Drop => QueuedInstruction::Dump(entity, inventory.drop_direction),
                InventoryAction::Wield => QueuedInstruction::Wield(entity),
                InventoryAction::Unwield => QueuedInstruction::Unwield(entity),
            };
            instruction_queue.add(instruction, InputChange::Held);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn remove_inventory_resource(mut commands: Commands) {
    commands.remove_resource::<InventoryScreen>();
}
