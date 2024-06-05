use super::{
    components::{QualitySituation, RecipeSituation},
    resource::CraftingScreen,
};
use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use std::{ops::RangeInclusive, time::Instant};

const MAX_FIND_DISTANCE: i32 = 7;
const FIND_RANGE: RangeInclusive<i32> = (-MAX_FIND_DISTANCE)..=MAX_FIND_DISTANCE;

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_crafting_screen(mut commands: Commands) {
    let recipe_list = commands
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
    let recipe_details = commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..default()
            },
            ..default()
        },))
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
            StateBound::<GameplayScreenState>::default(),
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
                                ..default()
                            },
                            ..default()
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
pub(super) fn clear_crafting_screen(
    clock: Clock,
    mut crafting_screen: ResMut<CraftingScreen>,
    children: Query<&Children>,
    mut styles: Query<&mut Style>,
) -> bool {
    if crafting_screen.last_time == clock.time() {
        return false;
    }
    crafting_screen.last_time = clock.time();
    crafting_screen.selection_list.clear(true);

    // TODO
    if let Ok(children) = children.get(crafting_screen.recipe_list) {
        for &child in children {
            if let Ok(mut style) = styles.get_mut(child) {
                style.display = Display::None;
            }
        }
    }

    true
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn update_crafting_screen(
    In(run): In<bool>,
    mut commands: Commands,
    location: Res<Location>,
    fonts: Res<Fonts>,
    infos: Res<Infos>,
    sav: Res<Sav>,
    mut crafting_screen: ResMut<CraftingScreen>,
    players: Query<(&Pos, &BodyContainers), With<Player>>,
    items_and_furniture: Query<(&ObjectDefinition, &Amount, &LastSeen, Option<&Parent>)>,
) {
    if !run {
        return;
    }

    let (&player_pos, body_containers) = players.single();

    let nearby_items = find_nearby(&location, &items_and_furniture, player_pos, body_containers);
    let nearby_manuals = nearby_manuals(&infos, &nearby_items);
    let nearby_qualities = nearby_qualities(&infos, &nearby_items);
    //println!("{:?}", &nearby_manuals);

    let shown_recipes = shown_recipes(&infos, &sav, &nearby_manuals, &nearby_qualities);

    let mut shown_qualities = nearby_qualities
        .iter()
        .map(|(quality_id, amount)| {
            (
                quality_id,
                amount,
                uppercase_first(
                    &infos
                        .quality(quality_id)
                        .expect("Quality should be found")
                        .name
                        .single,
                ),
            )
        })
        .collect::<Vec<_>>();
    shown_qualities.sort_by_key(|(.., name)| String::from(name));

    let mut first_recipe = None;
    commands
        .entity(crafting_screen.recipe_list)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                String::from("Known recipies:"),
                fonts.regular(WARN_TEXT_COLOR),
            ));

            for recipe in shown_recipes {
                let first = crafting_screen.selection_list.selected.is_none();
                if first {
                    first_recipe = Some(recipe.clone());
                }

                let entity = parent
                    .spawn((
                        TextBundle::from_section(
                            recipe.name.clone(),
                            fonts.regular(recipe.color(first)),
                        ),
                        recipe,
                    ))
                    .id();
                crafting_screen.selection_list.append(entity);
            }

            parent.spawn(TextBundle::from_section(
                String::from("\nNearby tools:"),
                fonts.regular(WARN_TEXT_COLOR),
            ));

            for (_, amount, name) in shown_qualities {
                parent.spawn(TextBundle::from_section(
                    format!("{amount} {name}"),
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
        );
    } else {
        commands
            .entity(crafting_screen.recipe_details)
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    String::from("No recipes known"),
                    fonts.regular(BAD_TEXT_COLOR),
                ));
            });
    }
}

fn find_nearby<'a>(
    location: &'a Location,
    items_and_furniture: &'a Query<(&ObjectDefinition, &Amount, &LastSeen, Option<&Parent>)>,
    player_pos: Pos,
    body_containers: &'a BodyContainers,
) -> Vec<(&'a ObjectDefinition, &'a Amount)> {
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
        .map(|(definition, amount, ..)| (definition, amount))
        .collect::<Vec<_>>()
}

fn nearby_manuals<'a>(
    infos: &'a Infos,
    nearby_items: &[(&'a ObjectDefinition, &'a Amount)],
) -> HashMap<&'a ObjectId, &'a str> {
    nearby_items
        .iter()
        .map(|(definition, _)| *definition)
        .filter(|definition| {
            definition.category == ObjectCategory::Item
                && infos
                    .item(&definition.id)
                    .filter(|item| item.category.as_ref().is_some_and(|s| s == "manuals"))
                    .is_some()
        })
        .map(|definition| &definition.id)
        .map(|manual_id| {
            (
                manual_id,
                infos
                    .item(manual_id)
                    .expect("Manual should be known")
                    .name
                    .single
                    .as_str(),
            )
        })
        .collect::<HashMap<_, _>>()
}

fn shown_recipes(
    infos: &Infos,
    sav: &Sav,
    nearby_manuals: &HashMap<&ObjectId, &str>,
    nearby_qualities: &HashMap<&ObjectId, i8>,
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
                .item(&recipe.result)
                .ok_or(0)
                .inspect_err(|_| {
                    eprintln!("Recipe result {:?} should be a known item", recipe.result);
                })
                .ok()
                .map(|item| RecipeSituation {
                    recipe_id: recipe_id.clone(),
                    name: uppercase_first(&item.name.single),
                    autolearn,
                    manuals: recipe_manuals,
                    qualities: recipe_qualities(infos, &recipe.qualities.0, nearby_qualities),
                })
        })
        .collect::<Vec<_>>();

    shown_recipes.sort_by_key(|recipe| (recipe.name.clone(), recipe.recipe_id.fallback_name()));
    shown_recipes
}

fn autolearn_recipe(recipe: &Recipe, skills: &HashMap<String, Skill>) -> bool {
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

fn recipe_manuals<'a>(
    recipe: &'a Recipe,
    nearby_manuals: &HashMap<&'a ObjectId, &'a str>,
) -> Vec<String> {
    // TODO check skill level

    let mut manuals = match &recipe.book_learn {
        BookLearn::List(list) => list.iter().map(BookLearnItem::id).collect::<Vec<_>>(),
        BookLearn::Map(map) => map.keys().collect::<Vec<_>>(),
        BookLearn::Other(other) => todo!("{other:?}"),
    }
    .iter()
    .filter_map(|from_book| nearby_manuals.get(from_book))
    .map(|name| String::from(*name))
    .collect::<Vec<_>>();

    manuals.sort();
    manuals
}

fn nearby_qualities<'a>(
    infos: &'a Infos,
    nearby_items: &[(&'a ObjectDefinition, &'a Amount)],
) -> HashMap<&'a ObjectId, i8> {
    let found = nearby_items
        .iter()
        .filter_map(|(definition, _)| match definition.category {
            ObjectCategory::Item => infos.item(&definition.id).map(|item| &item.qualities),
            ObjectCategory::Furniture => infos
                .furniture(&definition.id)
                .and_then(|furniture| furniture.crafting_pseudo_item.as_ref())
                .and_then(|pseude_item| infos.item(pseude_item).map(|item| &item.qualities)),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>();

    infos
        .qualities()
        .filter_map(|quality_id| {
            found
                .iter()
                .filter(|item_quality| &item_quality.0 == quality_id)
                .map(|item_quality| item_quality.1)
                .max()
                .map(|max| (quality_id, max))
        })
        .collect::<HashMap<_, _>>()
}

fn recipe_qualities(
    infos: &Infos,
    required: &[RequiredQuality],
    present: &HashMap<&ObjectId, i8>,
) -> Vec<QualitySituation> {
    let mut qualities = required
        .iter()
        .map(|required_quality| QualitySituation {
            name: uppercase_first(
                infos
                    .quality(&required_quality.id)
                    .expect("Quality should be known")
                    .name
                    .single
                    .as_str(),
            ),
            present: present.get(&required_quality.id).copied(),
            required: required_quality.level,
        })
        .collect::<Vec<_>>();

    qualities.sort_by_key(|quality| quality.name.clone());
    qualities
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_crafting_keyboard_input(
    mut commands: Commands,
    mut keys: Keys,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    infos: Res<Infos>,
    fonts: Res<Fonts>,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut crafting_screen: ResMut<CraftingScreen>,
    mut recipes: Query<(&mut Text, &Transform, &Node, &RecipeSituation)>,
    mut scrolling_lists: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    scrolling_parents: Query<(&Node, &Style), Without<ScrollingList>>,
) {
    let start = Instant::now();

    for combo in keys.combos(Ctrl::Without) {
        match combo.key {
            Key::Code(KeyCode::Escape) | Key::Character('&')
                if combo.change == InputChange::JustPressed =>
            {
                next_gameplay_state.set(GameplayScreenState::Base);
            }
            Key::Code(direction @ (KeyCode::ArrowUp | KeyCode::ArrowDown)) => {
                if direction == KeyCode::ArrowUp {
                    crafting_screen.select_up(&mut recipes);
                } else {
                    crafting_screen.select_down(&mut recipes);
                }
                if let Some(selected) = crafting_screen.selection_list.selected {
                    let (_, recipe_transform, recipe_node, recipe_sitation) = recipes
                        .get(selected)
                        .expect("Selected recipe should be found");

                    {
                        let (mut scrolling_list, mut style, parent, list_node) =
                            scrolling_lists.single_mut();
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
                        &mut commands,
                        &infos,
                        &fonts,
                        &crafting_screen,
                        recipe_sitation,
                    );
                }
            }
            _ => {}
        }
    }

    log_if_slow("manage_crafting_keyboard_input", start);
}

fn show_recipe(
    commands: &mut Commands,
    infos: &Infos,
    fonts: &Fonts,
    crafting_screen: &CraftingScreen,
    recipe_sitation: &RecipeSituation,
) {
    let recipe = infos
        .recipe(&recipe_sitation.recipe_id)
        .expect("Recipe id should esist");

    commands
        .entity(crafting_screen.recipe_details)
        .despawn_descendants()
        .with_children(|builder| {
            builder.spawn(TextBundle::from_sections(
                recipe_sitation.text_sections(fonts, recipe),
            ));
        });
}

/*#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_crafting_button_input(
    mut instruction_queue: ResMut<InstructionQueue>,
    interactions: Query<(&Interaction, &CraftingButton), (Changed<Interaction>, With<Button>)>,
    crafting_screen: Res<CraftingScreen>,
) {
    // TODO
}*/

#[allow(clippy::needless_pass_by_value)]
pub(super) fn remove_crafting_resource(mut commands: Commands) {
    commands.remove_resource::<CraftingScreen>();
}
