use crate::common::{log_if_slow, uppercase_first};
use crate::gameplay::screens::crafting::components::{
    AlternativeSituation, ComponentSituation, QualitySituation, RecipeSituation,
};
use crate::gameplay::screens::crafting::resource::CraftingScreen;
use crate::gameplay::{
    ActiveSav, Amount, BodyContainers, Clock, GameplayScreenState, Infos, InstructionQueue,
    LastSeen, Location, MessageWriter, ObjectCategory, ObjectDefinition, Player, Pos,
    QueuedInstruction,
};
use crate::hud::{
    ButtonBuilder, Fonts, ScrollingList, SelectionList, BAD_TEXT_COLOR, GOOD_TEXT_COLOR,
    PANEL_COLOR, SMALL_SPACING, WARN_TEXT_COLOR,
};
use crate::keyboard::{Held, Key, KeyBindings};
use crate::manual::ManualSection;
use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};
use cdda_json_files::{
    Alternative, AutoLearn, BookLearn, BookLearnItem, ObjectId, Recipe, RequiredQuality, Sav,
    Skill, Using,
};
use std::{ops::RangeInclusive, sync::Arc, time::Instant};
use units::Timestamp;

const MAX_FIND_DISTANCE: i32 = 7;
const FIND_RANGE: RangeInclusive<i32> = (-MAX_FIND_DISTANCE)..=MAX_FIND_DISTANCE;

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
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    ..Style::default()
                },
                ..NodeBundle::default()
            },
            ScrollingList::default(),
        ))
        .id();
    let recipe_details = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..Style::default()
            },
            ..NodeBundle::default()
        })
        .id();
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..Style::default()
                },
                ..NodeBundle::default()
            },
            StateScoped(GameplayScreenState::Crafting),
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
                        ..Style::default()
                    },
                    background_color: PANEL_COLOR.into(),
                    ..NodeBundle::default()
                })
                .with_children(|builder| {
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Start,
                                justify_content: JustifyContent::Start,
                                overflow: Overflow::clip_y(),
                                ..Style::default()
                            },
                            ..NodeBundle::default()
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
    infos: Res<Infos>,
    fonts: Res<Fonts>,
    mut crafting_screen: ResMut<CraftingScreen>,
    mut recipes: Query<(&mut TextStyle, &Transform, &Node, &RecipeSituation)>,
    mut scrolling_lists: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    scrolling_parents: Query<(&Node, &Style), Without<ScrollingList>>,
) {
    let start = Instant::now();

    let Key::Code(key_code) = key else {
        eprintln!("Unexpected key {key:?} while moving crafting selection");
        return;
    };

    crafting_screen.adjust_selection(&mut recipes.transmute_lens().query(), &key_code);
    adapt_to_selected(
        &mut commands,
        &infos,
        &fonts,
        &crafting_screen,
        &recipes.transmute_lens().query(),
        &mut scrolling_lists,
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
    mut styles: Query<&mut Style>,
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
    items_and_furniture: Query<(
        Entity,
        &ObjectDefinition,
        &Amount,
        &LastSeen,
        Option<&Parent>,
    )>,
) {
    let Some(start_craft_system) = start_craft_system else {
        return;
    };

    let (&player_pos, body_containers) = players.single();

    let nearby_items = find_nearby(&location, &items_and_furniture, player_pos, body_containers);
    let nearby_manuals = nearby_manuals(&infos, &nearby_items);
    let nearby_qualities = nearby_qualities(&infos, &nearby_items);
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
        .map(|(quality_id, amount)| {
            (
                quality_id,
                amount,
                uppercase_first(infos.quality(quality_id).name.single.clone()),
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
                fonts.regular(WARN_TEXT_COLOR),
            ));

            for recipe in shown_recipes {
                let first = crafting_screen.selection_list.selected.is_none();
                if first {
                    first_recipe = Some(recipe.clone());
                }

                let entity = parent
                    .spawn((
                        Text::from(&*recipe.name),
                        fonts.regular(recipe.color(first)),
                        recipe,
                    ))
                    .id();
                crafting_screen.selection_list.append(entity);
            }

            parent.spawn((
                Text::from("\nNearby tools:"),
                fonts.regular(WARN_TEXT_COLOR),
            ));

            for (_, amount, name) in shown_qualities {
                parent.spawn((
                    Text::from(format!("{amount} {name}")),
                    fonts.regular(GOOD_TEXT_COLOR),
                ));
            }
        });

    if let Some(first_recipe) = first_recipe {
        show_recipe(
            &mut commands,
            &infos,
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
                    fonts.regular(BAD_TEXT_COLOR),
                ));
            });
    }
}

fn find_nearby<'a>(
    location: &'a Location,
    items_and_furniture: &'a Query<(
        Entity,
        &ObjectDefinition,
        &Amount,
        &LastSeen,
        Option<&Parent>,
    )>,
    player_pos: Pos,
    body_containers: &'a BodyContainers,
) -> Vec<(Entity, &'a ObjectDefinition, &'a Amount)> {
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
        .map(|(entity, definition, amount, ..)| (entity, definition, amount))
        .collect::<Vec<_>>()
}

fn nearby_manuals<'a>(
    infos: &'a Infos,
    nearby_items: &[(Entity, &'a ObjectDefinition, &'a Amount)],
) -> HashMap<ObjectId, Arc<str>> {
    nearby_items
        .iter()
        .map(|(_, definition, _)| *definition)
        .filter(|definition| {
            definition.category == ObjectCategory::Item
                && infos
                    .try_item(&definition.id)
                    .filter(|item| item.category.as_ref().is_some_and(|s| &**s == "manuals"))
                    .is_some()
        })
        .map(|definition| {
            (
                definition.id.clone(),
                infos.item(&definition.id).name.single.clone(),
            )
        })
        .collect::<HashMap<_, _>>()
}

fn shown_recipes(
    infos: &Infos,
    sav: &Sav,
    nearby_manuals: &HashMap<ObjectId, Arc<str>>,
    nearby_qualities: &HashMap<ObjectId, i8>,
    nearby_items: &[(Entity, &ObjectDefinition, &Amount)],
) -> Vec<RecipeSituation> {
    let mut shown_recipes = infos
        .recipes()
        .map(|(recipe_id, recipe)| {
            (
                recipe_id,
                recipe,
                autolearn_recipe(recipe, &sav.player.skills),
                recipe_manuals(recipe, nearby_manuals),
            )
        })
        .filter(|(.., autolearn, recipe_manuals)| *autolearn || !recipe_manuals.is_empty())
        .filter_map(|(recipe_id, recipe, autolearn, recipe_manuals)| {
            infos
                .try_item(&recipe.result)
                .ok_or(0)
                .inspect_err(|_| {
                    eprintln!("Recipe result {:?} should be a known item", recipe.result);
                })
                .ok()
                .map(|item| RecipeSituation {
                    recipe_id: recipe_id.clone(),
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
            recipe.recipe_id.fallback_name(),
        )
    });
    shown_recipes
}

fn autolearn_recipe(recipe: &Recipe, skills: &HashMap<Arc<str>, Skill>) -> bool {
    match &recipe.autolearn {
        AutoLearn::Bool(autolearn) => {
            *autolearn
                && recipe.skill_used.as_ref().map_or(true, |skill_used| {
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

fn nearby_qualities<'a>(
    infos: &'a Infos,
    nearby_items: &[(Entity, &'a ObjectDefinition, &'a Amount)],
) -> HashMap<ObjectId, i8> {
    let found = nearby_items
        .iter()
        .filter_map(|(_, definition, _)| match definition.category {
            ObjectCategory::Item => infos.try_item(&definition.id).map(|item| &item.qualities),
            ObjectCategory::Furniture => infos
                .try_furniture(&definition.id)
                .and_then(|furniture| furniture.crafting_pseudo_item.as_ref())
                .and_then(|pseude_item| infos.try_item(pseude_item).map(|item| &item.qualities)),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>();

    infos
        .qualities()
        .filter_map(|quality_id| {
            found
                .iter()
                .filter(|item_quality| item_quality.0 == quality_id)
                .map(|item_quality| item_quality.1)
                .max()
                .map(|max| (quality_id, max))
        })
        .collect::<HashMap<_, _>>()
}

fn recipe_qualities(
    infos: &Infos,
    required: &[RequiredQuality],
    using: &[Using],
    present: &HashMap<ObjectId, i8>,
) -> Vec<QualitySituation> {
    let mut qualities = required
        .iter()
        .chain(
            using
                .iter()
                //.inspect(|using| println!("Using qualities from {using:?}"))
                .flat_map(|using| &infos.requirement(&using.requirement).qualities.0),
        )
        .map(|required_quality| QualitySituation {
            name: uppercase_first(infos.quality(&required_quality.id).name.single.clone()),
            present: present.get(&required_quality.id).copied(),
            required: required_quality.level,
        })
        .collect::<Vec<_>>();

    qualities.sort_by_key(|quality| quality.name.clone());
    qualities
}

fn recipe_components(
    infos: &Infos,
    required: &[Vec<Alternative>],
    using: &[Using],
    present: &[(Entity, &ObjectDefinition, &Amount)],
) -> Vec<ComponentSituation> {
    let using = using
        .iter()
        .flat_map(|using| infos.to_components(using))
        .collect::<Vec<_>>();

    required
        .iter()
        .chain(using.iter())
        .map(|component| ComponentSituation {
            alternatives: {
                let mut alternatives = component
                    .iter()
                    .flat_map(|alternative| expand_items(infos, alternative))
                    .filter_map(|(item_id, required)| {
                        infos
                            .try_item(item_id)
                            .ok_or(())
                            .inspect_err(|()| {
                                eprintln!("Item {item_id:?} not found (maybe comestible?)");
                            })
                            .ok()
                            .map(|item| (item_id, item, required))
                    })
                    .map(|(item_id, item, required)| {
                        let (item_entities, amounts): (Vec<_>, Vec<&Amount>) = present
                            .iter()
                            .filter(|(_, definition, _)| definition.id == *item_id)
                            .map(|(entity, _, amount)| (entity, amount))
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

fn expand_items<'a>(infos: &'a Infos, alternative: &'a Alternative) -> Vec<(&'a ObjectId, u32)> {
    match alternative {
        Alternative::Item { item, required } => vec![(item, *required)],
        Alternative::Requirement {
            requirement,
            factor,
        } => {
            let Some(requirement) = infos.try_requirement(requirement) else {
                assert!(
                    infos.try_item(requirement).is_some(),
                    "Unkonwn requirement {:?} should be an items",
                    &requirement
                );
                return vec![(requirement, *factor)];
            };
            if requirement.components.len() != 1 {
                eprintln!(
                    "Unexpected structure for {requirement:?}: {:?}",
                    &requirement.components
                );
            }

            requirement
                .components
                .iter()
                .flatten()
                .flat_map(|alternative| {
                    expand_items(infos, alternative)
                        .into_iter()
                        .map(|(item_id, amount)| (item_id, factor * amount))
                })
                .collect()
        }
    }
}

fn adapt_to_selected(
    commands: &mut Commands,
    infos: &Res<Infos>,
    fonts: &Res<Fonts>,
    crafting_screen: &CraftingScreen,
    recipes: &Query<(&Transform, &Node, &RecipeSituation)>,
    scrolling_lists: &mut Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    scrolling_parents: &Query<(&Node, &Style), Without<ScrollingList>>,
    start_craft_system: &StartCraftSystem,
) {
    if let Some(selected) = crafting_screen.selection_list.selected {
        let (recipe_transform, recipe_node, recipe_sitation) = recipes
            .get(selected)
            .expect("Selected recipe should be found");

        {
            let (mut scrolling_list, mut style, parent, list_node) = scrolling_lists
                .get_mut(crafting_screen.recipe_list)
                .expect("The recipe list should be a scrolling list");
            let (parent_node, parent_style) = scrolling_parents
                .get(parent.get())
                .expect("Parent node should be found");
            style.top = scrolling_list.follow(
                recipe_transform,
                recipe_node,
                list_node,
                parent_node,
                parent_style,
            );
        }

        show_recipe(
            commands,
            infos,
            fonts,
            crafting_screen,
            recipe_sitation,
            start_craft_system,
        );
    }
}

fn show_recipe(
    commands: &mut Commands,
    infos: &Infos,
    fonts: &Fonts,
    crafting_screen: &CraftingScreen,
    recipe_sitation: &RecipeSituation,
    start_craft_system: &StartCraftSystem,
) {
    let recipe = infos.recipe(&recipe_sitation.recipe_id);

    commands
        .entity(crafting_screen.recipe_details)
        .despawn_descendants()
        .with_children(|parent| {
            ButtonBuilder::new(
                "Craft",
                fonts.regular(recipe_sitation.color(true)),
                start_craft_system.0,
            )
            .spawn(parent, ());
            parent
                .spawn((Text::default(), fonts.standard()))
                .with_children(|parent| {
                    for section in recipe_sitation.text_sections(fonts, recipe) {
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
            .add(&*recipe.name)
            .send_error();
    }

    log_if_slow("start_craft", start);
}

pub(super) fn remove_crafting_resource(mut commands: Commands) {
    commands.remove_resource::<CraftingScreen>();
}
