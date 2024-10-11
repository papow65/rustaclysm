use crate::common::log_if_slow;
use crate::gameplay::screens::inventory::components::{
    InventoryAction, InventoryItemDescription, InventoryItemLine,
};
use crate::gameplay::screens::inventory::{resource::InventoryScreen, section::InventorySection};
use crate::gameplay::{
    BodyContainers, Clock, Corpse, Envir, GameplayScreenState, HorizontalDirection, Infos,
    InstructionQueue, Integrity, Item, LastSeen, Nbor, Phrase, Player, PlayerDirection, Pos,
    QueuedInstruction,
};
use crate::hud::{
    ButtonBuilder, Fonts, ScrollingList, SelectionList, StepDirection, StepSize, GOOD_TEXT_COLOR,
    HARD_TEXT_COLOR, PANEL_COLOR, SMALL_SPACING, SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
};
use crate::keyboard::{Held, Key, KeyBindings};
use crate::manual::ManualSection;
use bevy::ecs::{entity::EntityHashMap, system::SystemId};
use bevy::{prelude::*, utils::HashMap};
use cdda_json_files::{ItemInfo, ObjectId};
use std::time::Instant;
use units::Timestamp;

#[derive(Clone, Debug)]
pub(super) struct InventoryButton {
    pub(super) item: Entity,
    pub(super) action: InventoryAction,
}

#[derive(Debug)]
pub(super) struct InventorySystem(SystemId<In<InventoryButton>, ()>);

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_inventory_system(world: &mut World) -> InventorySystem {
    InventorySystem(world.register_system_cached(handle_inventory_action))
}

#[expect(clippy::needless_pass_by_value)]
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
        drop_direction: HorizontalDirection::Here,
        section_by_item: EntityHashMap::default(),
        section_text_style: fonts.regular(SOFT_TEXT_COLOR),
        drop_section_text_style: fonts.regular(WARN_TEXT_COLOR),
        item_text_style: fonts.regular(HARD_TEXT_COLOR),
        selected_item_text_style: fonts.regular(GOOD_TEXT_COLOR),
        last_time: Timestamp::ZERO,
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_inventory_key_bindings(
    world: &mut World,
    held_bindings: Local<KeyBindings<GameplayScreenState, (), Held>>,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    held_bindings.spawn(
        world,
        GameplayScreenState::Inventory,
        |bindings| {
            bindings.add_multi(
                [
                    KeyCode::ArrowUp,
                    KeyCode::ArrowDown,
                    KeyCode::PageUp,
                    KeyCode::PageDown,
                ],
                move_inventory_selection,
            );
        },
        ManualSection::new(
            &[
                ("select item", "arrow up/down"),
                ("select item", "page up/down"),
            ],
            100,
        ),
    );

    fresh_bindings.spawn(
        world,
        GameplayScreenState::Inventory,
        |bindings| {
            bindings.add_multi(
                [
                    KeyCode::Numpad1,
                    KeyCode::Numpad2,
                    KeyCode::Numpad3,
                    KeyCode::Numpad4,
                    KeyCode::Numpad5,
                    KeyCode::Numpad6,
                    KeyCode::Numpad7,
                    KeyCode::Numpad8,
                    KeyCode::Numpad9,
                ],
                set_inventory_drop_direction,
            );
            bindings.add_multi(['d', 't', 'u', 'w'], handle_selected_item);
            bindings.add('e', examine_selected_item);
            bindings.add_multi(
                [Key::Code(KeyCode::Escape), Key::Character('i')],
                exit_inventory,
            );
        },
        ManualSection::new(
            &[
                ("set drop spot", "numpad"),
                ("drop item", "d"),
                ("examine item", "e"),
                ("take item", "t"),
                ("wield item", "w"),
                ("unwield item", "u"),
                ("close inventory", "esc/i"),
            ],
            101,
        ),
    );

    log_if_slow("create_inventory_key_bindings", start);
}

#[expect(clippy::needless_pass_by_value)]
fn move_inventory_selection(
    In(key): In<Key>,
    mut commands: Commands,
    mut inventory: ResMut<InventoryScreen>,
    item_lines: Query<(&InventoryItemLine, &Children)>,
    item_texts: Query<(Entity, &InventoryItemDescription)>,
    item_buttons: Query<&Children, With<Button>>,
    mut text_styles: Query<&mut TextStyle>,
    item_layouts: Query<(&Transform, &Node)>,
    mut scrolling_lists: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    scrolling_parents: Query<(&Node, &Style), Without<ScrollingList>>,
) {
    let Key::Code(key_code) = key else {
        eprintln!("Unexpected key {key:?} while moving inventory selection");
        return;
    };

    inventory.adjust_selection(
        &mut commands,
        &item_lines,
        &item_texts,
        &item_buttons,
        &mut text_styles,
        &key_code,
    );
    follow_selected(
        &inventory,
        &item_layouts,
        &mut scrolling_lists,
        &scrolling_parents,
    );
}

fn set_inventory_drop_direction(In(key): In<Key>, mut inventory: ResMut<InventoryScreen>) {
    let Ok(player_direction) = PlayerDirection::try_from(key) else {
        eprintln!("Unexpected key {key:?} while setting inventory drop direction");
        return;
    };

    let Nbor::Horizontal(horizontal_direction) = player_direction.to_nbor() else {
        eprintln!(
            "Unexpected direction {player_direction:?} while setting inventory drop direction"
        );
        return;
    };

    inventory.drop_direction = horizontal_direction;
    inventory.last_time = Timestamp::ZERO;
}

fn exit_inventory(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    next_gameplay_state.set(GameplayScreenState::Base);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn clear_inventory(
    In(inventory_system): In<InventorySystem>,
    clock: Clock,
    mut inventory: ResMut<InventoryScreen>,
    children: Query<&Children>,
    mut styles: Query<&mut Style>,
) -> Option<InventorySystem> {
    if inventory.last_time == clock.time() {
        return None;
    }
    inventory.last_time = clock.time();

    if let Ok(children) = children.get(inventory.panel) {
        for &child in children {
            if let Ok(mut style) = styles.get_mut(child) {
                style.display = Display::None;
            }
        }
    }

    Some(inventory_system)
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn refresh_inventory(
    In(inventory_system): In<Option<InventorySystem>>,
    mut commands: Commands,
    envir: Envir,
    infos: Res<Infos>,
    mut inventory: ResMut<InventoryScreen>,
    players: Query<(&Pos, &BodyContainers), With<Player>>,
    items: Query<(Item, Option<&Corpse>, Option<&Integrity>, &LastSeen)>,
    previous_item_lines: Query<&InventoryItemLine>,
) {
    let Some(inventory_system) = inventory_system else {
        return;
    };

    let previous_selected_item = inventory
        .selection_list
        .selected
        .and_then(|row| previous_item_lines.get(row).ok())
        .map(|line| line.item)
        .filter(|item| items.contains(*item));
    println!("Refresh inventory, with previous selected {previous_selected_item:?}");
    inventory.selection_list.clear();

    let (&player_pos, body_containers) = players.single();
    let items_by_section = items_by_section(&envir, &items, player_pos, body_containers);
    let mut items_by_section = items_by_section.into_iter().collect::<Vec<_>>();
    items_by_section.sort_by_key(|(section, _)| *section);

    commands.entity(inventory.panel).with_children(|parent| {
        for (section, items) in items_by_section {
            let drop_section = section == InventorySection::Nbor(inventory.drop_direction);
            let section_style = if drop_section {
                &inventory.drop_section_text_style
            } else {
                &inventory.section_text_style
            };

            parent
                .spawn((Text::new(format!("[{section}]")), section_style.clone()))
                .with_children(|parent| {
                    if drop_section {
                        parent.spawn((TextSpan::from("(dropping here)"), section_style.clone()));
                    }
                });

            for (item_entity, id, item_phrase) in items {
                if let Some(item_info) = infos.try_item(id) {
                    let is_selected;
                    let is_selected_previous;
                    if let Some(previous_selected_item) = previous_selected_item {
                        is_selected = item_entity == previous_selected_item;
                        is_selected_previous = is_selected;
                    } else {
                        is_selected = inventory.selection_list.selected.is_none();
                        is_selected_previous = false;
                    }
                    let item_style = if is_selected {
                        &inventory.selected_item_text_style
                    } else {
                        &inventory.item_text_style
                    };

                    let row_entity = add_row(
                        &section,
                        parent,
                        item_entity,
                        &item_phrase,
                        item_info,
                        item_style,
                        drop_section,
                        &inventory_system,
                    );

                    inventory.selection_list.append(row_entity);
                    if is_selected_previous {
                        println!("Previous selected found");
                        inventory.selection_list.selected = Some(row_entity);
                    }
                    inventory.section_by_item.insert(item_entity, section);
                } else {
                    eprintln!("Unknown item: {id:?}");
                }
            }

            // empty line
            parent.spawn((Text::from(" "), inventory.section_text_style.clone()));
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
    item_entity: Entity,
    item_phrase: &Phrase,
    item_info: &ItemInfo,
    item_text_style: &TextStyle,
    drop_section: bool,
    inventory_system: &InventorySystem,
) -> Entity {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    column_gap: SMALL_SPACING,
                    ..default()
                },
                ..default()
            },
            InventoryItemLine { item: item_entity },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Text::default(),
                    item_text_style.clone(),
                    Style {
                        width: Val::Px(200.0),
                        overflow: Overflow::clip(),
                        ..Style::default()
                    },
                    InventoryItemDescription(item_phrase.clone()),
                ))
                .with_children(|parent| {
                    for section in item_phrase.as_text_sections(item_text_style) {
                        parent.spawn(section);
                    }
                });

            parent.spawn((
                Text::from(if let Some(ref volume) = item_info.volume {
                    format!("{volume}")
                } else {
                    String::new()
                }),
                item_text_style.clone(),
                Style {
                    width: Val::Px(60.0),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::End,
                    ..Style::default()
                },
            ));

            parent.spawn((
                Text::from(if let Some(ref mass) = item_info.mass {
                    format!("{mass}")
                } else {
                    String::new()
                }),
                item_text_style.clone(),
                Style {
                    width: Val::Px(60.0),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::End,
                    ..Style::default()
                },
            ));

            for action in actions(section, drop_section) {
                let caption = format!("{}", &action);
                ButtonBuilder::new(caption, item_text_style.clone(), inventory_system.0).spawn(
                    parent,
                    InventoryButton {
                        item: item_entity,
                        action,
                    },
                );
            }
        })
        .id()
}

fn actions(section: &InventorySection, drop_section: bool) -> Vec<InventoryAction> {
    let mut actions = vec![InventoryAction::Examine];
    if matches!(section, InventorySection::Nbor(_)) {
        actions.push(InventoryAction::Take);
        if !drop_section {
            actions.push(InventoryAction::Move);
        }
    } else {
        actions.push(InventoryAction::Drop);
    }
    if matches!(section, InventorySection::Hands) {
        actions.push(InventoryAction::Unwield);
    } else {
        actions.push(InventoryAction::Wield);
    }
    actions
}

fn follow_selected(
    inventory: &InventoryScreen,
    items: &Query<(&Transform, &Node)>,
    scrolling_lists: &mut Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    scrolling_parents: &Query<(&Node, &Style), Without<ScrollingList>>,
) {
    let Some(selected_row) = inventory.selection_list.selected else {
        return;
    };

    let (item_transform, item_node) = items
        .get(selected_row)
        .expect("Selected item should be found");

    let (mut scrolling_list, mut style, parent, list_node) = scrolling_lists
        .get_mut(inventory.panel)
        .expect("The inventory panel should be a scrolling list");
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

#[expect(clippy::needless_pass_by_value)]
fn handle_selected_item(
    In(key): In<Key>,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut inventory: ResMut<InventoryScreen>,
    item_lines: Query<&InventoryItemLine>,
) {
    let Key::Character(char) = key else {
        eprintln!("Unexpected key {key:?} while handling selected item");
        return;
    };

    if let Some(selected_item) = inventory.selected_item(&item_lines) {
        instruction_queue.add(match char {
            'd' => {
                let Some(item_section) = inventory.section_by_item.get(&selected_item) else {
                    eprintln!("Section of item {selected_item:?} not found");
                    return;
                };
                if &InventorySection::Nbor(inventory.drop_direction) == item_section {
                    // Prevent moving an item to its current position.
                    return;
                }
                QueuedInstruction::Dump(selected_item, inventory.drop_direction)
            }
            't' => QueuedInstruction::Pickup(selected_item),
            'u' => QueuedInstruction::Unwield(selected_item),
            'w' => QueuedInstruction::Wield(selected_item),
            _ => panic!("Unexpected key {char:?}"),
        });
        inventory
            .selection_list
            .adjust(StepSize::Single, StepDirection::Down);
    }
}

/// Special case, because we don't want to select another item after the action.
#[expect(clippy::needless_pass_by_value)]
fn examine_selected_item(
    mut instruction_queue: ResMut<InstructionQueue>,
    inventory: Res<InventoryScreen>,
    item_lines: Query<&InventoryItemLine>,
) {
    if let Some(selected_item) = inventory.selected_item(&item_lines) {
        instruction_queue.add(QueuedInstruction::ExamineItem(selected_item));
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn handle_inventory_action(
    In(inventory_button): In<InventoryButton>,
    mut instruction_queue: ResMut<InstructionQueue>,
    inventory: Res<InventoryScreen>,
) {
    println!("{:?}", &inventory_button);
    let instruction = match inventory_button.action {
        InventoryAction::Examine => QueuedInstruction::ExamineItem(inventory_button.item),
        InventoryAction::Take => QueuedInstruction::Pickup(inventory_button.item),
        InventoryAction::Drop | InventoryAction::Move => {
            QueuedInstruction::Dump(inventory_button.item, inventory.drop_direction)
        }
        InventoryAction::Wield => QueuedInstruction::Wield(inventory_button.item),
        InventoryAction::Unwield => QueuedInstruction::Unwield(inventory_button.item),
    };
    instruction_queue.add(instruction);
}

pub(super) fn remove_inventory_resource(mut commands: Commands) {
    commands.remove_resource::<InventoryScreen>();
}
