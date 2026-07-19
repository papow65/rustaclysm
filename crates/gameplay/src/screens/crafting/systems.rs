use crate::screens::crafting::{CraftingScreen, messages::YouLackTheMeansToCraft};
use bevy::ecs::{spawn::SpawnIter, system::SystemId};
use bevy::prelude::{
    Added, AnyOf, Children, Commands, DespawnOnExit, Entity, In, KeyCode, Local, NextState, Query,
    RemovedComponents, Res, ResMut, Single, SpawnRelated as _, Text, TextColor, With, World,
    children, debug,
};
use cdda_json_files::{FurnitureInfo, TerrainInfo};
use gameplay_action_planning::{PlayerInstructions, QueuedInstruction};
use gameplay_cdda::Infos;
use gameplay_cdda_active_sav::ActiveSav;
use gameplay_common::Shared;
use gameplay_crafting::{RecipeSituation, shown_recipes};
use gameplay_item::{BodyContainers, Item, ItemHierarchy};
use gameplay_location::{LocationCache, Pos};
use gameplay_log::LogMessageWriter;
use gameplay_model::LastSeen;
use gameplay_player::Player;
use gameplay_screen_state::GameplayScreenState;
use hud::{BAD_TEXT_COLOR, ButtonBuilder, WARN_TEXT_COLOR};
use keyboard::KeyBindings;
use manual::ManualSection;
use selection_list::{SelectableItemIn, SelectedItemIn, selection_list_detail_screen};
use std::time::Instant;
use util::log_if_slow;

#[derive(Debug)]
pub(super) struct StartCraftSystem(SystemId<(), ()>);

pub(super) fn create_start_craft_system(world: &mut World) -> StartCraftSystem {
    StartCraftSystem(world.register_system_cached(start_craft))
}

pub(super) fn spawn_crafting_screen(
    In(start_craft_system): In<StartCraftSystem>,
    mut commands: Commands,
) {
    let (recipe_list, recipe_details) =
        selection_list_detail_screen(&mut commands, GameplayScreenState::Crafting);

    commands.insert_resource(CraftingScreen::new(
        recipe_list,
        recipe_details,
        start_craft_system,
    ));
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn create_crafting_key_bindings(
    world: &mut World,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    fresh_bindings.spawn(world, GameplayScreenState::Crafting, |bindings| {
        bindings.add('c', start_craft);
        bindings.add(KeyCode::Escape, exit_crafting);
        bindings.add('&', exit_crafting);
    });

    world.spawn((
        ManualSection::new(&[("craft", "c"), ("close crafting", "esc/&")], 100),
        DespawnOnExit(GameplayScreenState::Crafting),
    ));

    log_if_slow("create_crafting_key_bindings", start);
}

fn exit_crafting(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    let start = Instant::now();

    next_gameplay_state.set(GameplayScreenState::Base);

    log_if_slow("exit_crafting", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn adapt_to_crafting_selection(
    mut commands: Commands,
    crafting_screen: Res<CraftingScreen>,
    mut selected_recipes: Query<(&mut TextColor, &RecipeSituation), Added<SelectedItemIn>>,
) {
    let start = Instant::now();

    for (mut text_color, recipe) in &mut selected_recipes {
        //debug!("Selected: {}", recipe.name);
        *text_color = recipe.color(true);

        show_recipe(&mut commands, &crafting_screen, recipe);
    }

    log_if_slow("adapt_to_crafting_selection", start);
}

pub(super) fn adapt_to_crafting_deselection(
    mut removed: RemovedComponents<SelectedItemIn>,
    mut recipes: Query<(&mut TextColor, &RecipeSituation)>,
) {
    let start = Instant::now();

    removed.read().for_each(|deselected_recipe| {
        let (text_color, recipe) = &mut recipes
            .get_mut(deselected_recipe)
            .expect("Previous highlighted recipe should be found");
        **text_color = recipe.color(false);
        //debug!("Deselected: {}", recipe.name);
    });

    log_if_slow("adapt_to_crafting_deselection", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn refresh_crafting_screen(
    mut commands: Commands,
    location: Res<LocationCache>,
    infos: Res<Infos>,
    active_sav: Res<ActiveSav>,
    crafting_screen: Res<CraftingScreen>,
    hierarchy: ItemHierarchy,
    player: Single<(&Pos, &BodyContainers), With<Player>>,
    items: Query<(Item, &LastSeen)>,
    infrastructure: Query<(
        AnyOf<(&Shared<FurnitureInfo>, &Shared<TerrainInfo>)>,
        &LastSeen,
    )>,
) {
    let (&player_pos, body_containers) = *player;

    let shown_recipes = shown_recipes(
        &location,
        &infos,
        active_sav.sav(),
        &hierarchy,
        player_pos,
        body_containers,
        &items,
        &infrastructure,
    );

    let mut recipe_entities = Vec::new();

    commands
        .entity(crafting_screen.recipe_list)
        .despawn_related::<Children>()
        .with_children(|parent| {
            parent.spawn((Text::from("Known recipies:"), WARN_TEXT_COLOR));

            for recipe in shown_recipes {
                let recipe_entity = parent.spawn(recipe.to_text_bundle()).id();
                recipe_entities.push(recipe_entity);
            }
        })
        .add_related::<SelectableItemIn>(&recipe_entities);

    commands
        .entity(crafting_screen.recipe_details)
        .with_children(|parent| {
            parent.spawn((Text::from("No recipes known"), BAD_TEXT_COLOR));
        });
}

fn show_recipe(
    commands: &mut Commands,
    crafting_screen: &CraftingScreen,
    recipe_sitation: &RecipeSituation,
) {
    commands
        .entity(crafting_screen.recipe_details)
        .despawn_related::<Children>()
        .insert(children![
            ButtonBuilder::new(
                "Craft",
                recipe_sitation.color(true),
                crafting_screen.start_craft_system().0,
                (),
            )
            .bundle(),
            (
                Text::default(),
                Children::spawn((SpawnIter(
                    recipe_sitation
                        .text_sections(recipe_sitation.recipe())
                        .into_iter()
                ),)),
            ),
        ]);
}

#[expect(clippy::needless_pass_by_value)]
fn start_craft(
    mut message_writer: LogMessageWriter,
    mut player_instructions: ResMut<PlayerInstructions>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    selected_item_in: Single<Entity, With<SelectedItemIn>>,
    recipes: Query<&RecipeSituation>,
) {
    let start = Instant::now();

    let selected_craft = *selected_item_in;
    let recipe = recipes
        .get(selected_craft)
        .expect("The selected craft should be found");
    if recipe.craftable() {
        debug!("Craft {recipe:?}");
        player_instructions.push(QueuedInstruction::StartCraft(recipe.clone()));
        // Close the crafting screen
        next_gameplay_state.set(GameplayScreenState::Base);
    } else {
        message_writer.send(YouLackTheMeansToCraft {
            recipe: recipe.name().clone(),
        });
    }

    log_if_slow("start_craft", start);
}

pub(super) fn remove_crafting_resource(mut commands: Commands) {
    commands.remove_resource::<CraftingScreen>();
}
