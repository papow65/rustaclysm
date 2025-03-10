use crate::gameplay::screens::crafting::components::{
    AlternativeSituation, ComponentSituation, QualitySituation, RecipeSituation,
};
use crate::gameplay::screens::crafting::resource::CraftingScreen;
use crate::gameplay::{
    ActiveSav, Amount, BodyContainers, Clock, GameplayScreenState, Info, Infos, InstructionQueue,
    LastSeen, Location, MessageWriter, ObjectCategory, ObjectDefinition, Player, Pos,
    QueuedInstruction, cdda::Error,
};
use crate::hud::{
    BAD_TEXT_COLOR, ButtonBuilder, Fonts, GOOD_TEXT_COLOR, PANEL_COLOR, SMALL_SPACING, ScrollList,
    SelectionList, WARN_TEXT_COLOR,
};
use crate::keyboard::{Held, Key, KeyBindings};
use crate::manual::ManualSection;
use crate::util::{log_if_slow, uppercase_first};
use bevy::prelude::{
    AlignItems, BuildChildren as _, ChildBuild as _, Children, Commands, ComputedNode,
    DespawnRecursiveExt as _, Display, Entity, FlexDirection, In, IntoSystem as _, JustifyContent,
    KeyCode, Local, NextState, Node, Overflow, Parent, Query, Res, ResMut, StateScoped, Text,
    TextColor, Transform, UiRect, Val, With, Without, World,
};
use bevy::utils::hashbrown::hash_map::Entry;
use bevy::{ecs::query::QueryData, ecs::system::SystemId, utils::HashMap};
use cdda_json_files::{
    Alternative, AutoLearn, BookLearn, BookLearnItem, CommonItemInfo, FurnitureInfo, ObjectId,
    Quality, Recipe, RequiredQuality, Sav, Skill, Using,
};
use std::{ops::RangeInclusive, sync::Arc, time::Instant};
use units::Timestamp;

const MAX_FIND_DISTANCE: i32 = 7;
const FIND_RANGE: RangeInclusive<i32> = (-MAX_FIND_DISTANCE)..=MAX_FIND_DISTANCE;

#[derive(QueryData)]
#[query_data(derive(Debug))]
pub(super) struct Nearby {
    entity: Entity,
    definition: &'static ObjectDefinition,
    amount: &'static Amount,
    common_item_info: Option<&'static Info<CommonItemInfo>>,
    furniture_info: Option<&'static Info<FurnitureInfo>>,
}

#[derive(Debug)]
pub(super) struct StartCraftSystem(SystemId<(), ()>);

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_start_craft_system(world: &mut World) -> StartCraftSystem {
    StartCraftSystem(world.register_system_cached(start_craft))
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_start_craft_system_with_key(
    In(key): In<Key>,
    world: &mut World,
) -> (Key, StartCraftSystem) {
    (key, create_start_craft_system(world))
}

pub(super) fn spawn_crafting_screen(mut commands: Commands) {
    let recipe_list = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..Node::default()
            },
            ScrollList::default(),
        ))
        .id();
    let recipe_details = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            ..Node::default()
        })
        .id();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            StateScoped(GameplayScreenState::Crafting),
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
                ))
                .with_children(|builder| {
                    builder
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Start,
                            justify_content: JustifyContent::Start,
                            overflow: Overflow::clip_y(),
                            ..Node::default()
                        })
                        .add_child(recipe_list)
                        .add_child(recipe_details);
                });
        });

    commands.insert_resource(CraftingScreen {
        recipe_list,
        selection_list: SelectionList::default(),
        recipe_details,
        last_time: Timestamp::ZERO,
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_crafting_key_bindings(
    world: &mut World,
    held_bindings: Local<KeyBindings<GameplayScreenState, (), Held>>,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    held_bindings.spawn(
        world,
        GameplayScreenState::Crafting,
        |bindings| {
            bindings.add_multi(
                [
                    KeyCode::ArrowUp,
                    KeyCode::ArrowDown,
                    KeyCode::PageUp,
                    KeyCode::PageDown,
                ],
                create_start_craft_system_with_key.pipe(move_crafting_selection),
            );
        },
        ManualSection::new(
            &[
                ("select craft", "arrow up/down"),
                ("select craft", "page up/down"),
            ],
            100,
        ),
    );

    fresh_bindings.spawn(
        world,
        GameplayScreenState::Crafting,
        |bindings| {
            bindings.add('c', start_craft);
            bindings.add_multi(
                [Key::Code(KeyCode::Escape), Key::Character('&')],
                exit_crafting,
            );
        },
        ManualSection::new(&[("craft", "c"), ("close crafting", "esc/&")], 100),
    );

    log_if_slow("create_crafting_key_bindings", start);
}

fn exit_crafting(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    let start = Instant::now();

    next_gameplay_state.set(GameplayScreenState::Base);

    log_if_slow("exit_crafting", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn move_crafting_selection(
    In((key, start_craft_system)): In<(Key, StartCraftSystem)>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    mut crafting_screen: ResMut<CraftingScreen>,
    mut recipes: Query<(&mut TextColor, &Transform, &ComputedNode, &RecipeSituation)>,
    mut scroll_lists: Query<(&mut ScrollList, &mut Node, &ComputedNode, &Parent)>,
    scrolling_parents: Query<(&Node, &ComputedNode), Without<ScrollList>>,
) {
    let start = Instant::now();

    let Key::Code(key_code) = key else {
        eprintln!("Unexpected key {key:?} while moving crafting selection");
        return;
    };

    crafting_screen.adjust_selection(&mut recipes.transmute_lens().query(), &key_code);
    adapt_to_selected(
        &mut commands,
        &fonts,
        &crafting_screen,
        &recipes.transmute_lens().query(),
        &mut scroll_lists,
        &scrolling_parents,
        &start_craft_system,
    );

    log_if_slow("move_crafting_selection", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn clear_crafting_screen(
    In(start_craft_system): In<StartCraftSystem>,
    clock: Clock,
    mut crafting_screen: ResMut<CraftingScreen>,
    children: Query<&Children>,
    mut styles: Query<&mut Node>,
) -> Option<StartCraftSystem> {
    if crafting_screen.last_time == clock.time() {
        return None;
    }
    crafting_screen.last_time = clock.time();
    crafting_screen.selection_list.clear();

    if let Ok(children) = children.get(crafting_screen.recipe_list) {
        for &child in children {
            if let Ok(mut style) = styles.get_mut(child) {
                style.display = Display::None;
            }
        }
    }

    Some(start_craft_system)
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn refresh_crafting_screen(
    In(start_craft_system): In<Option<StartCraftSystem>>,
    mut commands: Commands,
    location: Res<Location>,
    fonts: Res<Fonts>,
    infos: Res<Infos>,
    active_sav: Res<ActiveSav>,
    mut crafting_screen: ResMut<CraftingScreen>,
    players: Query<(&Pos, &BodyContainers), With<Player>>,
    items_and_furniture: Query<(Nearby, &LastSeen, Option<&Parent>)>,
) {
    let Some(start_craft_system) = start_craft_system else {
        return;
    };

    let (&player_pos, body_containers) = players.single();

    let nearby_items = find_nearby(&location, &items_and_furniture, player_pos, body_containers);
    let nearby_manuals = nearby_manuals(&nearby_items);
    let nearby_qualities = nearby_qualities(&nearby_items);
    //println!("{:?}", &nearby_manuals);

    let shown_recipes = shown_recipes(
        &infos,
        active_sav.sav(),
        &nearby_manuals,
        &nearby_qualities,
        &nearby_items,
    );

    let mut shown_qualities = nearby_qualities
        .iter()
        .map(|(quality, amount)| {
            (
                quality.id.clone(),
                amount,
                uppercase_first(quality.name.single.clone()),
            )
        })
        .collect::<Vec<_>>();
    shown_qualities.sort_by_key(|(.., name)| name.clone());

    let mut first_recipe = None;
    commands
        .entity(crafting_screen.recipe_list)
        .with_children(|parent| {
            parent.spawn((
                Text::from("Known recipies:"),
                WARN_TEXT_COLOR,
                fonts.regular(),
            ));

            for recipe in shown_recipes {
                let first = crafting_screen.selection_list.selected.is_none();
                if first {
                    first_recipe = Some(recipe.clone());
                }

                let entity = parent
                    .spawn((
                        Text::from(&*recipe.name),
                        recipe.color(first),
                        fonts.regular(),
                        recipe,
                    ))
                    .id();
                crafting_screen.selection_list.append(entity);
            }

            parent.spawn((
                Text::from("\nNearby tools:"),
                WARN_TEXT_COLOR,
                fonts.regular(),
            ));

            for (_, amount, name) in shown_qualities {
                parent.spawn((
                    Text::from(format!("{amount} {name}")),
                    GOOD_TEXT_COLOR,
                    fonts.regular(),
                ));
            }
        });

    if let Some(first_recipe) = first_recipe {
        show_recipe(
            &mut commands,
            &fonts,
            &crafting_screen,
            &first_recipe,
            &start_craft_system,
        );
    } else {
        commands
            .entity(crafting_screen.recipe_details)
            .with_children(|parent| {
                parent.spawn((
                    Text::from("No recipes known"),
                    BAD_TEXT_COLOR,
                    fonts.regular(),
                ));
            });
    }
}

fn find_nearby<'a>(
    location: &'a Location,
    items_and_furniture: &'a Query<(Nearby, &LastSeen, Option<&Parent>)>,
    player_pos: Pos,
    body_containers: &'a BodyContainers,
) -> Vec<NearbyItem<'a>> {
    FIND_RANGE
        .flat_map(move |dz| {
            FIND_RANGE.flat_map(move |dx| {
                location
                    .all(player_pos.horizontal_offset(dx, dz))
                    .filter_map(|entity| items_and_furniture.get(*entity).ok())
                    .filter(|(.., last_seen, _)| **last_seen != LastSeen::Never)
            })
        })
        .chain(items_and_furniture.iter().filter(|(.., parent)| {
            parent.is_some_and(|p| {
                p.get() == body_containers.hands || p.get() == body_containers.clothing
            })
        }))
        .map(|(nearby, ..)| nearby)
        .collect::<Vec<_>>()
}

fn nearby_manuals(nearby_items: &[NearbyItem]) -> HashMap<ObjectId, Arc<str>> {
    nearby_items
        .iter()
        .filter(|nearby| {
            nearby.definition.category == ObjectCategory::Item
                && nearby
                    .common_item_info
                    .filter(|item| item.category.as_ref().is_some_and(|s| &**s == "manuals"))
                    .is_some()
        })
        .filter_map(|nearby| {
            nearby
                .common_item_info
                .map(|item_info| (nearby.definition.id.clone(), item_info.name.single.clone()))
        })
        .collect::<HashMap<_, _>>()
}

fn shown_recipes(
    infos: &Infos,
    sav: &Sav,
    nearby_manuals: &HashMap<ObjectId, Arc<str>>,
    nearby_qualities: &HashMap<Arc<Quality>, i8>,
    nearby_items: &[NearbyItem],
) -> Vec<RecipeSituation> {
    let mut shown_recipes = infos
        .recipes()
        .map(|recipe| {
            (
                recipe,
                autolearn_recipe(recipe, &sav.player.skills),
                recipe_manuals(recipe, nearby_manuals),
            )
        })
        .filter(|(.., autolearn, recipe_manuals)| *autolearn || !recipe_manuals.is_empty())
        .filter_map(|(recipe, autolearn, recipe_manuals)| {
            recipe
                .result
                .get_or(|error| eprintln!("Unknown recipe result: {error:#?}"))
                .map(|item| RecipeSituation {
                    recipe: recipe.clone(),
                    name: uppercase_first(item.name.single.clone()),
                    autolearn,
                    manuals: recipe_manuals,
                    qualities: recipe_qualities(
                        infos,
                        &recipe.qualities.0,
                        &recipe.using,
                        nearby_qualities,
                    ),
                    components: recipe_components(
                        infos,
                        &recipe.components,
                        &recipe.using,
                        nearby_items,
                    ),
                })
        })
        .collect::<Vec<_>>();

    shown_recipes.sort_by_key(|recipe| {
        (
            !recipe.craftable(),
            recipe.name.clone(),
            recipe.recipe.id.fallback_name(),
        )
    });
    shown_recipes
}

fn autolearn_recipe(recipe: &Recipe, skills: &HashMap<Arc<str>, Skill>) -> bool {
    match &recipe.autolearn {
        AutoLearn::Bool(autolearn) => {
            *autolearn
                && recipe.skill_used.as_ref().is_none_or(|skill_used| {
                    recipe.difficulty
                        <= skills
                            .get(skill_used)
                            .unwrap_or_else(|| panic!("Skill {:?} not found", recipe.skill_used))
                            .level
                })
        }
        AutoLearn::Skills(autolearn_skills) => {
            autolearn_skills.iter().all(|(skill_name, skill_level)| {
                *skill_level
                    <= skills
                        .get(skill_name)
                        .unwrap_or_else(|| panic!("Skill {:?} not found", recipe.skill_used))
                        .level
            })
        }
    }
}

fn recipe_manuals(recipe: &Recipe, nearby_manuals: &HashMap<ObjectId, Arc<str>>) -> Vec<Arc<str>> {
    // TODO check skill level

    let mut manuals = match &recipe.book_learn {
        BookLearn::List(list) => list.iter().map(BookLearnItem::id).collect::<Vec<_>>(),
        BookLearn::Map(map) => map.keys().cloned().collect::<Vec<_>>(),
        BookLearn::Other(other) => todo!("{other:?}"),
    }
    .iter()
    .filter_map(|from_book| nearby_manuals.get(from_book))
    .cloned()
    .collect::<Vec<_>>();

    manuals.sort();
    manuals
}

fn nearby_qualities(nearby_items: &[NearbyItem]) -> HashMap<Arc<Quality>, i8> {
    nearby_items
        .iter()
        .filter_map(|nearby| match nearby.definition.category {
            ObjectCategory::Item => nearby.common_item_info.map(|item| item.qualities.get_all()),
            ObjectCategory::Furniture => nearby.furniture_info.and_then(|furniture| {
                furniture
                    .crafting_pseudo_item
                    .get()
                    .map(|item| item.qualities.get_all())
            }),
            _ => None,
        })
        .flatten()
        .fold(
            HashMap::default(),
            |mut map: HashMap<Arc<Quality>, i8>, (quality, amount)| {
                match map.entry(quality) {
                    Entry::Occupied(occ) => {
                        *occ.into_mut() = (*occ.get()).max(amount);
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(amount);
                    }
                };
                map
            },
        )
}

fn recipe_qualities(
    infos: &Infos,
    required: &[RequiredQuality],
    using: &[Using],
    present: &HashMap<Arc<Quality>, i8>,
) -> Vec<QualitySituation> {
    let mut qualities = required
        .iter()
        .chain(
            using
                .iter()
                .filter_map(|using| {
                    infos
                        .requirement(&using.requirement)
                        .inspect_err(|error| eprintln!("Requirement not found: {error:#?}"))
                        .ok()
                })
                //.inspect(|using| println!("Using qualities from {using:?}"))
                .flat_map(|requirement| &requirement.qualities.0),
        )
        .filter_map(|required_quality| {
            required_quality
                .quality
                .get_or(|error| eprintln!("Quality not found: {error:#?}"))
                .map(|quality| QualitySituation {
                    name: uppercase_first(quality.name.single.clone()),
                    present: present.get(&quality).copied(),
                    required: required_quality.level,
                })
        })
        .collect::<Vec<_>>();

    qualities.sort_by_key(|quality| quality.name.clone());
    qualities
}

fn recipe_components(
    infos: &Infos,
    required: &[Vec<Alternative>],
    using: &[Using],
    present: &[NearbyItem],
) -> Vec<ComponentSituation> {
    let using = using
        .iter()
        .filter_map(|using| {
            infos
                .to_components(using)
                .inspect_err(|error| eprintln!("Using not found: {error:#?}"))
                .ok()
        })
        .flatten()
        .collect::<Vec<_>>();

    required
        .iter()
        .chain(using.iter())
        .map(|component| ComponentSituation {
            alternatives: {
                let mut alternatives = component
                    .iter()
                    .filter_map(|alternative| {
                        expand_items(infos, alternative)
                            .inspect_err(|error| {
                                eprintln!(
                                    "Could not process alternative {alternative:?}: {error:#?}"
                                );
                            })
                            .ok()
                    })
                    .flatten()
                    .filter_map(|(item_id, required)| {
                        infos
                            .common_item_info(item_id)
                            .inspect_err(|error| {
                                eprintln!(
                                    "Item {item_id:?} not found (maybe comestible?): {error:#?}"
                                );
                            })
                            .ok()
                            .map(|item| (item_id, item, required))
                    })
                    .map(|(item_id, item, required)| {
                        let (item_entities, amounts): (Vec<_>, Vec<&Amount>) = present
                            .iter()
                            .filter(|nearby| nearby.definition.id == *item_id)
                            .map(|nearby| (nearby.entity, nearby.amount))
                            .unzip();
                        AlternativeSituation {
                            id: item_id.clone(),
                            name: item.name.amount(required).clone(),
                            required,
                            present: amounts.iter().map(|amount| amount.0).sum(),
                            item_entities,
                        }
                    })
                    .collect::<Vec<_>>();
                alternatives.sort_by_key(|alternative| !alternative.is_present());
                alternatives
            },
        })
        .collect::<Vec<_>>()
}

fn expand_items<'a>(
    infos: &'a Infos,
    alternative: &'a Alternative,
) -> Result<Vec<(&'a ObjectId, u32)>, Error> {
    match alternative {
        Alternative::Item { item, required } => Ok(vec![(item, *required)]),
        Alternative::Requirement {
            requirement,
            factor,
        } => {
            let requirement = match infos.requirement(requirement) {
                Ok(requirement) => requirement,
                Err(_req_error) => match infos.common_item_info(requirement) {
                    Ok(_) => return Ok(vec![(requirement, *factor)]),
                    Err(_item_error) => {
                        return Err(Error::UnexpectedRequirement {
                            _id: requirement.clone(),
                        });
                    }
                },
            };

            if requirement.components.len() != 1 {
                eprintln!(
                    "Unexpected structure for {requirement:?}: {:?}",
                    &requirement.components
                );
            }

            Ok(requirement
                .components
                .iter()
                .flatten()
                .flat_map(|alternative| {
                    expand_items(infos, alternative)
                        .inspect_err(|error| eprintln!("Could not expand: {error:#?}"))
                })
                .flat_map(|expanded| {
                    expanded
                        .into_iter()
                        .map(|(item_id, amount)| (item_id, factor * amount))
                })
                .collect())
        }
    }
}

fn adapt_to_selected(
    commands: &mut Commands,
    fonts: &Res<Fonts>,
    crafting_screen: &CraftingScreen,
    recipes: &Query<(&Transform, &ComputedNode, &RecipeSituation)>,
    scroll_lists: &mut Query<(&mut ScrollList, &mut Node, &ComputedNode, &Parent)>,
    scrolling_parents: &Query<(&Node, &ComputedNode), Without<ScrollList>>,
    start_craft_system: &StartCraftSystem,
) {
    if let Some(selected) = crafting_screen.selection_list.selected {
        let (recipe_transform, recipe_computed_node, recipe_sitation) = recipes
            .get(selected)
            .expect("Selected recipe should be found");

        {
            let (mut scroll_list, mut style, list_computed_node, parent) = scroll_lists
                .get_mut(crafting_screen.recipe_list)
                .expect("The recipe list should be a scrolling list");
            let (parent_node, parent_computed_node) = scrolling_parents
                .get(parent.get())
                .expect("Parent node should be found");
            style.top = scroll_list.follow(
                recipe_transform,
                recipe_computed_node,
                list_computed_node,
                parent_node,
                parent_computed_node,
            );
        }

        show_recipe(
            commands,
            fonts,
            crafting_screen,
            recipe_sitation,
            start_craft_system,
        );
    }
}

fn show_recipe(
    commands: &mut Commands,
    fonts: &Fonts,
    crafting_screen: &CraftingScreen,
    recipe_sitation: &RecipeSituation,
    start_craft_system: &StartCraftSystem,
) {
    commands
        .entity(crafting_screen.recipe_details)
        .despawn_descendants()
        .with_children(|parent| {
            ButtonBuilder::new(
                "Craft",
                recipe_sitation.color(true),
                fonts.regular(),
                start_craft_system.0,
            )
            .spawn(parent, ());
            parent
                .spawn((Text::default(), fonts.regular()))
                .with_children(|parent| {
                    for section in recipe_sitation.text_sections(fonts, &recipe_sitation.recipe) {
                        parent.spawn(section);
                    }
                });
        });
}

#[expect(clippy::needless_pass_by_value)]
fn start_craft(
    mut message_writer: MessageWriter,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut instruction_queue: ResMut<InstructionQueue>,
    crafting_screen: Res<CraftingScreen>,
    recipes: Query<&RecipeSituation>,
) {
    let start = Instant::now();

    let selected_craft = crafting_screen
        .selection_list
        .selected
        .expect("There should be a selected craft");
    let recipe = recipes
        .get(selected_craft)
        .expect("The selected craft should be found");
    if recipe.craftable() {
        println!("Craft {recipe:?}");
        instruction_queue.add(QueuedInstruction::StartCraft(recipe.clone()));
        // Close the crafting screen
        next_gameplay_state.set(GameplayScreenState::Base);
    } else {
        message_writer
            .you("lack the means to craft")
            .hard(&*recipe.name)
            .send_error();
    }

    log_if_slow("start_craft", start);
}

pub(super) fn remove_crafting_resource(mut commands: Commands) {
    commands.remove_resource::<CraftingScreen>();
}
