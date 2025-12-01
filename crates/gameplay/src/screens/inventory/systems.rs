use crate::screens::inventory::{
    InventoryAction, InventoryItemRow, InventoryScreen, InventorySection, RowSpawner,
};
use crate::{
    BodyContainers, DebugTextShown, Envir, ExamineItem, GameplayScreenState, ItemHierarchy,
    ItemItem, MoveItem, Phrase, Pickup, Player, PlayerInstructions, QueuedInstruction, Unwield,
    Wield,
};
use bevy::ecs::{entity::hash_map::EntityHashMap, system::SystemId};
use bevy::platform::collections::HashMap;
use bevy::prelude::{
    Added, BackgroundColor, Button, Children, Commands, DespawnOnExit, Entity, In, IntoSystem as _,
    KeyCode, Local, NextState, Query, RemovedComponents, Res, ResMut, Single, Text, TextColor,
    TextSpan, With, World, debug, error,
};
use gameplay_location::{HorizontalDirection, Nbor, Pos};
use hud::{Fonts, HARD_TEXT_COLOR, SOFT_TEXT_COLOR, scroll_screen};
use keyboard::KeyBindings;
use manual::ManualSection;
use selection_list::{SelectableItemIn, SelectedItemIn, SelectionListItems, SelectionListStep};
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
    let start = Instant::now();

    let panel = scroll_screen(&mut commands, GameplayScreenState::Inventory);

    commands.insert_resource(InventoryScreen::new(
        panel,
        HorizontalDirection::Here,
        EntityHashMap::default(),
        Timestamp::ZERO,
        inventory_system,
    ));

    log_if_slow("spawn_inventory", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn create_inventory_key_bindings(
    world: &mut World,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

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
        DespawnOnExit(GameplayScreenState::Inventory),
    ));

    log_if_slow("create_inventory_key_bindings", start);
}

pub(super) fn adapt_to_item_selection(
    selected_rows: Query<Entity, Added<SelectedItemIn>>,
    mut item_rows: Query<(&mut BackgroundColor, &Children)>,
    item_buttons: Query<&Children, With<Button>>,
    mut text_styles: Query<&mut TextColor>,
) {
    let start = Instant::now();

    for selected_row in &selected_rows {
        InventoryScreen::highlight_selected(
            selected_row,
            &mut item_rows,
            &item_buttons,
            &mut text_styles,
            true,
        );
    }

    log_if_slow("adapt_to_item_selection", start);
}

pub(super) fn adapt_to_item_deselection(
    mut removed: RemovedComponents<SelectedItemIn>,
    mut item_rows: Query<(&mut BackgroundColor, &Children)>,
    item_buttons: Query<&Children, With<Button>>,
    mut text_styles: Query<&mut TextColor>,
) {
    let start = Instant::now();

    removed.read().for_each(|deselected_row| {
        //warn!("Deselected item: {deselected_row:?}");
        InventoryScreen::highlight_selected(
            deselected_row,
            &mut item_rows,
            &item_buttons,
            &mut text_styles,
            false,
        );
    });

    log_if_slow("adapt_to_item_deselection", start);
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
pub(super) fn refresh_inventory(
    mut commands: Commands,
    fonts: Res<Fonts>,
    envir: Envir,
    debug_text_shown: Res<DebugTextShown>,
    item_hierarchy: ItemHierarchy,
    mut inventory: ResMut<InventoryScreen>,
    player: Single<(&Pos, &BodyContainers), With<Player>>,
    previous_item_row: Option<Single<&InventoryItemRow, With<SelectedItemIn>>>,
) {
    let start = Instant::now();

    let previous_selected_item = previous_item_row
        .map(|previous| (*previous).item)
        .filter(|item| item_hierarchy.exists(*item));
    debug!("Refresh inventory, with previous selected {previous_selected_item:?}");

    let (&player_pos, body_containers) = *player;
    let items_by_section = items_by_section(&envir, &item_hierarchy, player_pos, body_containers);
    let mut items_by_section = items_by_section.into_iter().collect::<Vec<_>>();
    items_by_section.sort_by_key(|(section, _)| *section);

    let inventory = &mut *inventory;
    let drop_direction = inventory.drop_direction;
    let mut item_entities = Vec::new();
    let mut selected = None;

    let mut panel = commands.entity(inventory.panel);
    panel
        .despawn_related::<Children>()
        .with_children(|parent| {
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
                    &mut inventory.section_by_item,
                    parent,
                    previous_selected_item,
                    section,
                    drop_section,
                    &mut item_entities,
                    &mut selected,
                );
                item_hierarchy.walk(&mut row_spawner, items);

                // empty row
                parent.spawn((Text::from(" "), SOFT_TEXT_COLOR, fonts.regular()));
            }
        })
        .add_related::<SelectableItemIn>(&item_entities);

    if let Some(selected) = selected {
        panel.add_one_related::<SelectedItemIn>(selected);
    }

    log_if_slow("refresh_inventory", start);
}

fn items_by_section<'i>(
    envir: &'i Envir,
    item_hierarchy: &'i ItemHierarchy,
    player_pos: Pos,
    body_containers: &'i BodyContainers,
) -> HashMap<InventorySection, Vec<ItemItem<'i, 'i>>> {
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
    mut commands: Commands,
    mut player_instructions: ResMut<PlayerInstructions>,
    inventory: Res<InventoryScreen>,
    selection_list: Single<(Entity, &SelectionListItems)>,
    selected_row: Option<Single<(Entity, &InventoryItemRow), With<SelectedItemIn>>>,
) {
    let start = Instant::now();

    let Some(selected_row) = selected_row else {
        return;
    };
    let (selected_row_entity, selected_row) = *selected_row;
    let selected_item = selected_row.item;
    let (selection_list_entity, selection_list) = *selection_list;

    let next_row_entity = selection_list.offset(selected_row_entity, SelectionListStep::SingleDown);

    player_instructions.push(match action {
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
        commands
            .entity(selection_list_entity)
            .add_one_related::<SelectedItemIn>(next_row_entity);
    }

    log_if_slow("handle_selected_item", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn handle_inventory_action(
    In(inventory_button): In<InventoryButton>,
    mut player_instructions: ResMut<PlayerInstructions>,
    inventory: Res<InventoryScreen>,
) {
    let start = Instant::now();

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
    player_instructions.push(instruction);

    log_if_slow("handle_inventory_action", start);
}

pub(super) fn remove_inventory_resource(mut commands: Commands) {
    commands.remove_resource::<InventoryScreen>();
}
