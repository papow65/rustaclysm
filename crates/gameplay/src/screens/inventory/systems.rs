use crate::screens::inventory::components::{InventoryAction, InventoryItemRow};
use crate::screens::inventory::resource::InventoryScreen;
use crate::screens::inventory::row_spawner::RowSpawner;
use crate::screens::inventory::section::InventorySection;
use crate::{
    BodyContainers, Clock, DebugTextShown, Envir, ExamineItem, GameplayScreenState,
    InstructionQueue, ItemHierarchy, ItemItem, MoveItem, Phrase, Pickup, Player, QueuedInstruction,
    Unwield, Wield,
};
use bevy::ecs::{entity::hash_map::EntityHashMap, system::SystemId};
use bevy::picking::Pickable;
use bevy::platform::collections::HashMap;
use bevy::prelude::{
    AlignItems, BackgroundColor, Button, Children, Commands, Display, Entity, FlexDirection, In,
    IntoSystem as _, JustifyContent, KeyCode, Local, NextState, Node, Overflow, Query, Res, ResMut,
    Single, StateScoped, Text, TextColor, TextSpan, UiRect, Val, With, World, debug, error,
};
use gameplay_location::{HorizontalDirection, Nbor, Pos};
use hud::{
    Fonts, HARD_TEXT_COLOR, PANEL_COLOR, SMALL_SPACING, SOFT_TEXT_COLOR, SelectionList,
    SelectionListStep, scroll_to_selection,
};
use keyboard::{Held, KeyBindings};
use manual::{LargeNode, ManualSection};
use std::time::Instant;
use strum::VariantArray as _;
use units::Timestamp;
use util::log_if_slow;

#[derive(Clone, Debug)]
pub(super) struct InventoryButton {
    pub(super) item: Entity,
    pub(super) action: InventoryAction,
}

#[derive(Debug)]
pub(super) struct InventorySystem(pub(super) SystemId<In<InventoryButton>, ()>);

pub(super) fn create_inventory_system(world: &mut World) -> InventorySystem {
    InventorySystem(world.register_system_cached(handle_inventory_action))
}

pub(super) fn spawn_inventory(In(inventory_system): In<InventorySystem>, mut commands: Commands) {
    let panel = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                overflow: Overflow::scroll_y(),
                ..Node::default()
            },
            SelectionList::default(),
            Pickable::default(),
        ))
        .id();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            StateScoped(GameplayScreenState::Inventory),
            Pickable::IGNORE,
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Auto,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        margin: UiRect::px(10.0, 365.0, 10.0, 10.0),
                        padding: UiRect::all(SMALL_SPACING),
                        overflow: Overflow::clip_y(),
                        ..Node::default()
                    },
                    PANEL_COLOR,
                    LargeNode,
                    Pickable::IGNORE,
                ))
                .add_child(panel);
        });

    commands.insert_resource(InventoryScreen::new(
        panel,
        HorizontalDirection::Here,
        EntityHashMap::default(),
        Timestamp::ZERO,
        inventory_system,
    ));
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn create_inventory_key_bindings(
    world: &mut World,
    held_bindings: Local<KeyBindings<GameplayScreenState, (), Held>>,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    held_bindings.spawn(world, GameplayScreenState::Inventory, |bindings| {
        for &step in SelectionListStep::VARIANTS {
            bindings.add(
                step,
                (move || step)
                    .pipe(move_inventory_selection)
                    .pipe(scroll_to_selection),
            );
        }
    });

    world.spawn((
        ManualSection::new(
            &[
                ("select item", "arrow up/down"),
                ("select item", "page up/down"),
            ],
            100,
        ),
        StateScoped(GameplayScreenState::Inventory),
    ));

    fresh_bindings.spawn(world, GameplayScreenState::Inventory, |bindings| {
        for &horizontal_direction in HorizontalDirection::VARIANTS {
            bindings.add(
                Nbor::Horizontal(horizontal_direction),
                (move || horizontal_direction).pipe(set_inventory_drop_direction),
            );
        }
        for &inventory_action in InventoryAction::VARIANTS {
            bindings.add(
                inventory_action,
                (move || inventory_action).pipe(handle_selected_item),
            );
        }
        bindings.add(KeyCode::Escape, exit_inventory);
        bindings.add('i', exit_inventory);
    });

    world.spawn((
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
        StateScoped(GameplayScreenState::Inventory),
    ));

    log_if_slow("create_inventory_key_bindings", start);
}

#[expect(clippy::needless_pass_by_value)]
fn move_inventory_selection(
    In(step): In<SelectionListStep>,
    inventory: Res<InventoryScreen>,
    mut selection_lists: Query<&mut SelectionList>,
    mut item_rows: Query<(&InventoryItemRow, &mut BackgroundColor, &Children)>,
    item_buttons: Query<&Children, With<Button>>,
    mut text_styles: Query<&mut TextColor>,
) -> Entity {
    let start = Instant::now();

    let mut selection_list = selection_lists
        .get_mut(inventory.panel)
        .expect("Inventory selection list should be found");

    selection_list.adjust(step);

    if let Some(previous) = selection_list.previous_selected {
        InventoryScreen::highlight_selected(
            previous,
            &mut item_rows,
            &item_buttons,
            &mut text_styles,
            false,
        );
    }

    if let Some(selected) = selection_list.selected {
        InventoryScreen::highlight_selected(
            selected,
            &mut item_rows,
            &item_buttons,
            &mut text_styles,
            true,
        );
    }

    log_if_slow("move_crafting_selection", start);

    inventory.panel
}

fn set_inventory_drop_direction(
    In(horizontal_direction): In<HorizontalDirection>,
    mut inventory: ResMut<InventoryScreen>,
) {
    inventory.drop_direction = horizontal_direction;
    inventory.last_time = Timestamp::ZERO;
}

fn exit_inventory(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    next_gameplay_state.set(GameplayScreenState::Base);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn clear_inventory(
    clock: Clock,
    mut inventory: ResMut<InventoryScreen>,
    children: Query<&Children>,
    mut styles: Query<&mut Node>,
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

#[expect(clippy::needless_pass_by_value)]
pub(super) fn refresh_inventory(
    In(run): In<bool>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    envir: Envir,
    debug_text_shown: Res<DebugTextShown>,
    item_hierarchy: ItemHierarchy,
    mut inventory: ResMut<InventoryScreen>,
    player: Single<(&Pos, &BodyContainers), With<Player>>,
    mut selection_lists: Query<&mut SelectionList>,
    previous_item_rows: Query<&InventoryItemRow>,
) {
    if !run {
        return;
    }

    let mut selection_list = selection_lists
        .get_mut(inventory.panel)
        .expect("Inventory selection list should be found");

    let previous_selected_item = selection_list
        .selected
        .and_then(|row| previous_item_rows.get(row).ok())
        .map(|row| row.item)
        .filter(|item| item_hierarchy.exists(*item));
    debug!("Refresh inventory, with previous selected {previous_selected_item:?}");
    selection_list.clear();

    let (&player_pos, body_containers) = *player;
    let items_by_section = items_by_section(&envir, &item_hierarchy, player_pos, body_containers);
    let mut items_by_section = items_by_section.into_iter().collect::<Vec<_>>();
    items_by_section.sort_by_key(|(section, _)| *section);

    let inventory = &mut *inventory;
    let drop_direction = inventory.drop_direction;
    commands.entity(inventory.panel).with_children(|parent| {
        for (section, items) in items_by_section {
            let drop_section = section == InventorySection::Nbor(drop_direction);

            parent
                .spawn((
                    Text::new(format!("{section}")),
                    SOFT_TEXT_COLOR,
                    fonts.regular(),
                ))
                .with_children(|parent| {
                    if drop_section {
                        parent.spawn((
                            TextSpan::from(" <- drop spot"),
                            HARD_TEXT_COLOR,
                            fonts.regular(),
                        ));
                    }
                });

            let mut row_spawner = RowSpawner::new(
                &fonts,
                &debug_text_shown,
                &inventory.inventory_system,
                &mut selection_list,
                &mut inventory.section_by_item,
                parent,
                previous_selected_item,
                section,
                drop_section,
            );
            item_hierarchy.walk(&mut row_spawner, items);

            // empty row
            parent.spawn((Text::from(" "), SOFT_TEXT_COLOR, fonts.regular()));
        }
    });
}

fn items_by_section<'i>(
    envir: &'i Envir,
    item_hierarchy: &'i ItemHierarchy,
    player_pos: Pos,
    body_containers: &'i BodyContainers,
) -> HashMap<InventorySection, Vec<ItemItem<'i>>> {
    let mut items_by_section = HashMap::default();
    for (direction, nbor_pos) in envir.directions_for_item_handling(player_pos) {
        items_by_section.insert(
            InventorySection::Nbor(direction),
            envir.all_items(nbor_pos).collect::<Vec<_>>(),
        );
    }
    items_by_section.insert(
        InventorySection::Hands,
        item_hierarchy
            .items_in_pocket(body_containers.hands)
            .collect::<Vec<_>>(),
    );
    items_by_section.insert(
        InventorySection::Clothing,
        item_hierarchy
            .items_in_pocket(body_containers.clothing)
            .collect::<Vec<_>>(),
    );

    for items in items_by_section.values_mut() {
        items.sort_by_key(|item| Phrase::from_fragments(item.fragments().collect()).as_string());
    }

    items_by_section
}

#[expect(clippy::needless_pass_by_value)]
fn handle_selected_item(
    In(action): In<InventoryAction>,
    mut instruction_queue: ResMut<InstructionQueue>,
    inventory: Res<InventoryScreen>,
    mut selection_lists: Query<&mut SelectionList>,
    item_rows: Query<&InventoryItemRow>,
) {
    let mut selection_list = selection_lists
        .get_mut(inventory.panel)
        .expect("Inventory selection list should be found");

    let Some(selected_row) = selection_list.selected else {
        return;
    };
    let selected_row = item_rows
        .get(selected_row)
        .expect("Selected item row should be found");
    let selected_item = selected_row.item;

    instruction_queue.add(match action {
        InventoryAction::Examine => QueuedInstruction::ExamineItem(ExamineItem {
            item_entity: selected_item,
        }),
        InventoryAction::Drop | InventoryAction::Move => {
            let Some(item_section) = inventory.section_by_item.get(&selected_item) else {
                error!("Section of item {selected_item:?} not found");
                return;
            };
            if &InventorySection::Nbor(inventory.drop_direction) == item_section {
                // Prevent moving an item to its current position.
                return;
            }
            QueuedInstruction::MoveItem(MoveItem {
                item_entity: selected_item,
                to: Nbor::Horizontal(inventory.drop_direction),
            })
        }
        InventoryAction::Take => QueuedInstruction::Pickup(Pickup {
            item_entity: selected_item,
        }),
        InventoryAction::Unwield => QueuedInstruction::Unwield(Unwield {
            item_entity: selected_item,
        }),
        InventoryAction::Wield => QueuedInstruction::Wield(Wield {
            item_entity: selected_item,
        }),
    });

    if action != InventoryAction::Examine {
        selection_list.adjust(SelectionListStep::SingleDown);
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn handle_inventory_action(
    In(inventory_button): In<InventoryButton>,
    mut instruction_queue: ResMut<InstructionQueue>,
    inventory: Res<InventoryScreen>,
) {
    debug!("{:?}", &inventory_button);
    let item_entity = inventory_button.item;
    let instruction = match inventory_button.action {
        InventoryAction::Examine => QueuedInstruction::ExamineItem(ExamineItem { item_entity }),
        InventoryAction::Take => QueuedInstruction::Pickup(Pickup { item_entity }),
        InventoryAction::Drop | InventoryAction::Move => QueuedInstruction::MoveItem(MoveItem {
            item_entity,
            to: Nbor::Horizontal(inventory.drop_direction),
        }),
        InventoryAction::Wield => QueuedInstruction::Wield(Wield { item_entity }),
        InventoryAction::Unwield => QueuedInstruction::Unwield(Unwield { item_entity }),
    };
    instruction_queue.add(instruction);
}

pub(super) fn remove_inventory_resource(mut commands: Commands) {
    commands.remove_resource::<InventoryScreen>();
}
