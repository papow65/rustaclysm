use crate::screens::crafting::{
    AlternativeSituation, ComponentSituation, DetectedQuantity, QualitySituation, RecipeSituation,
    ToolSituation, resource::CraftingScreen,
};
use crate::screens::{
    Nearby, NearbyItem, find_nearby, find_nearby_pseudo, find_sources, nearby_qualities,
};
use crate::{
    ActiveSav, BodyContainers, Clock, GameplayScreenState, Infos, InstructionQueue, ItemHierarchy,
    LastSeen, Location, MessageWriter, Player, Pos, QueuedInstruction, Shared, cdda::Error,
};
use bevy::ecs::{spawn::SpawnIter, system::SystemId};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{
    AlignItems, AnyOf, ChildOf, Children, Commands, Display, Entity, FlexDirection, In,
    IntoSystem as _, JustifyContent, KeyCode, Local, NextState, Node, Overflow, Pickable, Query,
    Res, ResMut, Single, SpawnRelated as _, StateScoped, Text, TextColor, UiRect, Val, With, World,
    children, debug, error,
};
use cdda_json_files::{
    Alternative, AutoLearn, BookLearn, BookLearnItem, CalculatedRequirement, CommonItemInfo,
    FurnitureInfo, InfoId, PocketType, Quality, Recipe, RequiredComponent, RequiredPart,
    RequiredQuality, RequiredTool, Requirement, Sav, Skill, TerrainInfo,
};
use hud::{
    BAD_TEXT_COLOR, ButtonBuilder, Fonts, PANEL_COLOR, SMALL_SPACING, SelectionList,
    SelectionListStep, WARN_TEXT_COLOR, scroll_to_selection,
};
use keyboard::{Held, KeyBindings};
use manual::{LargeNode, ManualSection};
use std::num::NonZeroU32;
use std::{sync::Arc, time::Instant};
use strum::VariantArray as _;
use units::Timestamp;
use util::{log_if_slow, uppercase_first};

#[derive(Debug)]
pub(super) struct StartCraftSystem(SystemId<(), ()>);

pub(super) fn create_start_craft_system(world: &mut World) -> StartCraftSystem {
    StartCraftSystem(world.register_system_cached(start_craft))
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_crafting_screen(
    In(start_craft_system): In<StartCraftSystem>,
    mut commands: Commands,
    fonts: Res<Fonts>,
) {
    let recipe_list = commands
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
    let recipe_details = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            overflow: Overflow::scroll_y(),
            ..Node::default()
        })
        .id();
    commands
        .spawn((
            // Entire screen
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            StateScoped(GameplayScreenState::Crafting),
            Pickable::IGNORE,
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    // Panel
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        margin: UiRect::px(10.0, 365.0, 10.0, 10.0),
                        padding: UiRect::all(SMALL_SPACING),
                        ..Node::default()
                    },
                    PANEL_COLOR,
                    LargeNode,
                    Pickable::IGNORE,
                ))
                .with_children(|builder| {
                    builder
                        .spawn(
                            // Panel contents
                            (
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Start,
                                    justify_content: JustifyContent::Start,
                                    overflow: Overflow::clip_y(),
                                    ..Node::default()
                                },
                                Pickable::IGNORE,
                            ),
                        )
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    // Left column
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Start,
                                        justify_content: JustifyContent::Start,
                                        overflow: Overflow::clip_y(),
                                        ..Node::default()
                                    },
                                    Pickable::IGNORE,
                                    children![(
                                        Text::from("Known recipies:"),
                                        WARN_TEXT_COLOR,
                                        fonts.regular(),
                                    )],
                                ))
                                .add_child(recipe_list);
                        })
                        .add_child(recipe_details);
                });
        });

    commands.insert_resource(CraftingScreen::new(
        recipe_list,
        recipe_details,
        Timestamp::ZERO,
        start_craft_system,
    ));
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn create_crafting_key_bindings(
    world: &mut World,
    held_bindings: Local<KeyBindings<GameplayScreenState, (), Held>>,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    held_bindings.spawn(world, GameplayScreenState::Crafting, |bindings| {
        for &step in SelectionListStep::VARIANTS {
            bindings.add(
                step,
                (move || step)
                    .pipe(move_crafting_selection)
                    .pipe(scroll_to_selection),
            );
        }
    });

    world.spawn((
        ManualSection::new(
            &[
                ("select craft", "arrow up/down"),
                ("select craft", "page up/down"),
            ],
            100,
        ),
        StateScoped(GameplayScreenState::Crafting),
    ));

    fresh_bindings.spawn(world, GameplayScreenState::Crafting, |bindings| {
        bindings.add('c', start_craft);
        bindings.add(KeyCode::Escape, exit_crafting);
        bindings.add('&', exit_crafting);
    });

    world.spawn((
        ManualSection::new(&[("craft", "c"), ("close crafting", "esc/&")], 100),
        StateScoped(GameplayScreenState::Crafting),
    ));

    log_if_slow("create_crafting_key_bindings", start);
}

fn exit_crafting(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    let start = Instant::now();

    next_gameplay_state.set(GameplayScreenState::Base);

    log_if_slow("exit_crafting", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn move_crafting_selection(
    In(step): In<SelectionListStep>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    crafting_screen: Res<CraftingScreen>,
    mut recipes: Query<(&mut TextColor, &RecipeSituation)>,
    mut selection_lists: Query<&mut SelectionList>,
) -> Entity {
    let start = Instant::now();

    let mut selection_list = selection_lists
        .get_mut(crafting_screen.recipe_list)
        .expect("Recipe selection list should be found");

    selection_list.adjust(step);

    if let Some(previous) = selection_list.previous_selected {
        let (text_color, recipe) = &mut recipes
            .get_mut(previous)
            .expect("Previous highlighted recipe should ba found");
        **text_color = recipe.color(false);
    }

    if let Some(selected) = selection_list.selected {
        let (text_color, recipe) = &mut recipes
            .get_mut(selected)
            .expect("Highlighted recipe should ba found");
        **text_color = recipe.color(true);

        show_recipe(&mut commands, &fonts, &crafting_screen, recipe);
    }

    log_if_slow("move_crafting_selection", start);

    crafting_screen.recipe_list
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn clear_crafting_screen(
    clock: Clock,
    mut crafting_screen: ResMut<CraftingScreen>,
    mut selection_lists: Query<&mut SelectionList>,
    children: Query<&Children>,
    mut styles: Query<&mut Node>,
) -> bool {
    if crafting_screen.last_time == clock.time() {
        return false;
    }

    let mut selection_list = selection_lists
        .get_mut(crafting_screen.recipe_list)
        .expect("Recipe selection list should be found");

    crafting_screen.last_time = clock.time();
    selection_list.clear();

    if let Ok(children) = children.get(crafting_screen.recipe_list) {
        for &child in children {
            if let Ok(mut style) = styles.get_mut(child) {
                style.display = Display::None;
            }
        }
    }

    true
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn refresh_crafting_screen(
    In(run): In<bool>,
    mut commands: Commands,
    location: Res<Location>,
    fonts: Res<Fonts>,
    infos: Res<Infos>,
    active_sav: Res<ActiveSav>,
    crafting_screen: Res<CraftingScreen>,
    mut selection_lists: Query<&mut SelectionList>,
    hierarchy: ItemHierarchy,
    player: Single<(&Pos, &BodyContainers), With<Player>>,
    items: Query<(Nearby, &LastSeen, Option<&ChildOf>)>,
    infrastructure: Query<(
        AnyOf<(&Shared<FurnitureInfo>, &Shared<TerrainInfo>)>,
        &LastSeen,
    )>,
) {
    if !run {
        return;
    }

    let (&player_pos, body_containers) = *player;

    let nearby_items = find_nearby(&location, &items, player_pos, body_containers);
    let nearby_pseudo_items = find_nearby_pseudo(&location, &infrastructure, player_pos);
    let nearby_sources = find_sources(&location, &infrastructure, player_pos);
    let nearby_manuals = nearby_manuals(&nearby_items);
    let nearby_qualities = nearby_qualities(&nearby_items, &nearby_pseudo_items);
    //trace!("{:?}", &nearby_manuals);

    let shown_recipes = shown_recipes(
        &infos,
        active_sav.sav(),
        &hierarchy,
        &nearby_manuals,
        &nearby_qualities,
        &nearby_items,
        &nearby_pseudo_items,
        &nearby_sources,
    );

    let mut selection_list = selection_lists
        .get_mut(crafting_screen.recipe_list)
        .expect("Recipe selection list should be found");

    let mut first_recipe = None;
    commands
        .entity(crafting_screen.recipe_list)
        .with_children(|parent| {
            for recipe in shown_recipes {
                let first = selection_list.selected.is_none();
                if first {
                    first_recipe = Some(recipe.clone());
                }

                let entity = parent
                    .spawn((
                        Text::from(&*recipe.name),
                        recipe.color(first),
                        fonts.regular(),
                        recipe,
                        Pickable::IGNORE,
                    ))
                    .id();
                selection_list.append(entity);
            }
        });

    if let Some(first_recipe) = first_recipe {
        show_recipe(&mut commands, &fonts, &crafting_screen, &first_recipe);
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

fn nearby_manuals(nearby_items: &[NearbyItem]) -> HashMap<InfoId<CommonItemInfo>, Arc<str>> {
    nearby_items
        .iter()
        .filter(|nearby| {
            nearby
                .common_item_info
                .category
                .as_ref()
                .is_some_and(|s| &**s == "manuals")
        })
        .map(|nearby| {
            (
                nearby.common_item_info.id.clone(),
                nearby.common_item_info.name.single.clone(),
            )
        })
        .collect::<HashMap<_, _>>()
}

fn shown_recipes(
    infos: &Infos,
    sav: &Sav,
    hierarchy: &ItemHierarchy,
    nearby_manuals: &HashMap<InfoId<CommonItemInfo>, Arc<str>>,
    nearby_qualities: &HashMap<Arc<Quality>, i8>,
    nearby_items: &[NearbyItem],
    nearby_pseudo: &HashSet<Arc<CommonItemInfo>>,
    nearby_sources: &HashSet<InfoId<CommonItemInfo>>,
) -> Vec<RecipeSituation> {
    let mut shown_recipes = infos
        .recipes
        .values()
        .map(|recipe| {
            (
                recipe,
                autolearn_recipe(recipe, &sav.player.skills),
                recipe_manuals(recipe, nearby_manuals),
            )
        })
        .filter(|(.., autolearn, recipe_manuals)| *autolearn || !recipe_manuals.is_empty())
        .filter_map(|(recipe, autolearn, recipe_manuals)| {
            recipe.result.item_info().and_then(|result| {
                CalculatedRequirement::try_from(&**recipe)
                    .inspect_err(|error| error!("{error:?}"))
                    .ok()
                    .map(|calculated_requirements| RecipeSituation {
                        recipe: recipe.clone(),
                        name: uppercase_first(result.name.single.clone()),
                        autolearn,
                        manuals: recipe_manuals,
                        qualities: recipe_qualities(
                            &calculated_requirements.qualities.0,
                            nearby_qualities,
                        ),
                        tools: recipe_tools(
                            hierarchy,
                            &calculated_requirements.tools,
                            nearby_items,
                            nearby_pseudo,
                            nearby_sources,
                        ),
                        components: recipe_components(
                            &calculated_requirements.components,
                            nearby_items,
                            nearby_sources,
                        ),
                    })
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

fn recipe_manuals(
    recipe: &Recipe,
    nearby_manuals: &HashMap<InfoId<CommonItemInfo>, Arc<str>>,
) -> Vec<Arc<str>> {
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

fn recipe_qualities(
    required: &[RequiredQuality],
    present: &HashMap<Arc<Quality>, i8>,
) -> Vec<QualitySituation> {
    let mut qualities = required
        .iter()
        .filter_map(|required_quality| {
            required_quality
                .quality
                .get_option()
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

fn recipe_tools(
    hierarchy: &ItemHierarchy,
    required: &[Vec<Alternative<RequiredTool>>],
    present: &[NearbyItem],
    nearby_pseudo: &HashSet<Arc<CommonItemInfo>>,
    sources: &HashSet<InfoId<CommonItemInfo>>,
) -> Vec<ToolSituation> {
    required
        .iter()
        .map(|tool| ToolSituation {
            alternatives: expand_alternatives::<RequiredTool, _, _>(
                tool,
                |requirement| &requirement.tools,
                |item_info| {
                    let source = sources.contains(&item_info.id);

                    let pseudo = nearby_pseudo.iter().any(|pseudo| pseudo.id == item_info.id);

                    let mut found = present
                        .iter()
                        .filter(|nearby| nearby.common_item_info.id == item_info.id)
                        .peekable();
                    if !source && !pseudo && found.peek().is_none() {
                        return DetectedQuantity::Missing;
                    }

                    let (from_entities, amounts): (Vec<_>, Vec<_>) = found
                        .flat_map(|nearby| {
                            hierarchy.pockets_in(nearby.entity).filter_map(
                                |(pocket_entity, pocket)| match pocket.type_ {
                                    PocketType::Magazine => Some(pocket_entity),
                                    PocketType::MagazineWell => hierarchy
                                        .items_in(pocket_entity)
                                        .flat_map(|magazine| {
                                            hierarchy.pockets_in(magazine.entity).filter_map(
                                                |(pocket_entity, pocket)| match pocket.type_ {
                                                    PocketType::Magazine => Some(pocket_entity),
                                                    _ => None,
                                                },
                                            )
                                        })
                                        .next(),
                                    _ => None,
                                },
                            )
                        })
                        .flat_map(|pocket_entity| hierarchy.items_in(pocket_entity))
                        .map(|magazine_item| (magazine_item.entity, magazine_item.amount.0))
                        .unzip();

                    let total = amounts.into_iter().sum::<u32>();
                    if let Ok(present) = NonZeroU32::try_from(total) {
                        DetectedQuantity::Limited {
                            present: RequiredTool::from(present),
                            from_entities,
                        }
                    } else {
                        DetectedQuantity::Limited {
                            present: RequiredTool::present(),
                            from_entities,
                        }
                    }
                },
            ),
        })
        .collect::<Vec<_>>()
}

fn recipe_components(
    required: &[Vec<Alternative<RequiredComponent>>],
    present: &[NearbyItem],
    sources: &HashSet<InfoId<CommonItemInfo>>,
) -> Vec<ComponentSituation> {
    required
        .iter()
        .map(|component| ComponentSituation {
            alternatives: expand_alternatives::<RequiredComponent, _, _>(
                component,
                |requirement| &requirement.components,
                |item_info| {
                    if sources.contains(&item_info.id) {
                        return DetectedQuantity::Infinite;
                    }

                    let (from_entities, amounts): (Vec<_>, Vec<_>) = present
                        .iter()
                        .filter(|nearby| nearby.common_item_info.id == item_info.id)
                        .map(|nearby| (nearby.entity, nearby.amount.0))
                        .unzip();

                    let total = amounts.into_iter().sum::<u32>();
                    if let Ok(amount) = NonZeroU32::try_from(total) {
                        DetectedQuantity::Limited {
                            present: RequiredComponent { amount },
                            from_entities,
                        }
                    } else {
                        DetectedQuantity::Missing
                    }
                },
            ),
        })
        .collect::<Vec<_>>()
}

fn expand_alternatives<
    R: RequiredPart<Output = R>,
    S: Clone + Copy + Fn(&Requirement) -> &Vec<Vec<Alternative<R>>>,
    T: Clone + Copy + Fn(&Arc<CommonItemInfo>) -> DetectedQuantity<R>,
>(
    alternatives: &[Alternative<R>],
    details: S,
    present: T,
) -> Vec<AlternativeSituation<R>> {
    let mut alternatives = alternatives
        .iter()
        .filter_map(|alternative| {
            expand_items(alternative, details)
                .inspect_err(|error| {
                    error!("Could not process alternative {alternative:?}: {error:#?}");
                })
                .ok()
        })
        .flatten()
        .map(|(item_info, required)| AlternativeSituation {
            id: item_info.id.clone(),
            name: item_info.name.amount(required.item_amount()).clone(),
            required,
            detected: present(&item_info),
        })
        .collect::<Vec<_>>();
    alternatives.sort_by_key(|alternative| !alternative.is_present());
    alternatives
}

fn expand_items<
    R: RequiredPart,
    P: Clone + Copy + Fn(&Requirement) -> &Vec<Vec<Alternative<R>>>,
>(
    alternative: &Alternative<R>,
    details: P,
) -> Result<Vec<(Arc<CommonItemInfo>, R)>, Error> {
    match alternative {
        Alternative::Item { item, required, .. } => Ok(vec![(item.get()?, *required)]),
        Alternative::Requirement {
            requirement,
            factor,
        } => {
            let requirement = requirement.get()?;
            if details(&requirement).len() != 1 {
                error!(
                    "Unexpected tools or components ({:?}) in {requirement:#?}",
                    &requirement.components
                );
            }

            Ok(details(&requirement)
                .iter()
                .flatten()
                .flat_map(|alternative| {
                    expand_items(alternative, details)
                        .inspect_err(|error| error!("Could not expand: {error:#?}"))
                })
                .flat_map(|expanded| {
                    expanded
                        .into_iter()
                        .map(|(item_id, amount)| (item_id, *factor * amount))
                })
                .collect())
        }
    }
}

fn show_recipe(
    commands: &mut Commands,
    fonts: &Fonts,
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
                fonts.regular(),
                crafting_screen.start_craft_system().0,
                (),
            )
            .bundle(),
            (
                Text::default(),
                fonts.regular(),
                Children::spawn((SpawnIter(
                    recipe_sitation
                        .text_sections(fonts, &recipe_sitation.recipe)
                        .into_iter()
                ),)),
            ),
        ]);
}

#[expect(clippy::needless_pass_by_value)]
fn start_craft(
    mut message_writer: MessageWriter,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut instruction_queue: ResMut<InstructionQueue>,
    crafting_screen: Res<CraftingScreen>,
    selection_lists: Query<&SelectionList>,
    recipes: Query<&RecipeSituation>,
) {
    let start = Instant::now();

    let selection_list = selection_lists
        .get(crafting_screen.recipe_list)
        .expect("Recipe selection list should be found");
    let selected_craft = selection_list
        .selected
        .expect("There should be a selected craft");
    let recipe = recipes
        .get(selected_craft)
        .expect("The selected craft should be found");
    if recipe.craftable() {
        debug!("Craft {recipe:?}");
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
