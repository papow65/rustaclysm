use super::resource::CraftingScreen;
use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use std::ops::RangeInclusive;

const MAX_FIND_DISTANCE: i32 = 7;
const FIND_RANGE: RangeInclusive<i32> = (-MAX_FIND_DISTANCE)..=MAX_FIND_DISTANCE;

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_crafting_screen(mut commands: Commands) {
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
                .add_child(panel);
        });

    commands.insert_resource(CraftingScreen {
        panel,
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

    if let Ok(children) = children.get(crafting_screen.panel) {
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
    crafting_screen: Res<CraftingScreen>,
    players: Query<(&Pos, &BodyContainers), With<Player>>,
    items_and_furniture: Query<(&ObjectDefinition, &Amount, &LastSeen, Option<&Parent>)>,
) {
    if !run {
        return;
    }

    let (&player_pos, body_containers) = players.single();

    let nearby_items = find_nearby(&location, &items_and_furniture, player_pos, body_containers);
    let nearby_qualities = nearby_qualities(&infos, &nearby_items);
    let nearby_manuals = nearby_manuals(&infos, &nearby_items);
    //println!("{:?}", &nearby_manuals);

    let mut shown_recipes = infos
        .recipes()
        .filter(|(_, recipe)| known_recipe(recipe, &nearby_manuals, &sav.player.skills))
        .filter_map(|(recipe_id, recipe)| {
            infos
                .item(&recipe.result)
                .ok_or(0)
                .inspect_err(|_| {
                    eprintln!("Recipe result {:?} should be a known item", recipe.result);
                })
                .ok()
                .map(|item| {
                    (
                        recipe_id,
                        recipe,
                        uppercase_first(&item.name.single),
                        if qualities_present(&recipe.qualities.0, &nearby_qualities) {
                            DEFAULT_TEXT_COLOR
                        } else {
                            SOFT_TEXT_COLOR
                        },
                    )
                })
        })
        .collect::<Vec<_>>();
    shown_recipes.sort_by_key(|(.., name, _)| String::from(name));

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

    commands
        .entity(crafting_screen.panel)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                String::from("Known recipies:"),
                fonts.regular(WARN_TEXT_COLOR),
            ));

            for (.., name, color) in shown_recipes {
                parent.spawn(TextBundle::from_section(name, fonts.regular(color)));
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
) -> Vec<&'a ObjectId> {
    nearby_items
        .iter()
        .map(|(definition, _)| definition)
        .filter(|definition| {
            definition.category == ObjectCategory::Item
                && infos
                    .item(&definition.id)
                    .filter(|item| item.category.as_ref().is_some_and(|s| s == "manuals"))
                    .is_some()
        })
        .map(|definition| &definition.id)
        .collect::<Vec<_>>()
}

fn known_recipe(
    recipe: &Recipe,
    nearby_manuals: &[&ObjectId],
    skills: &HashMap<String, Skill>,
) -> bool {
    if let BookLearn::List(list) = &recipe.book_learn {
        list.iter()
            .any(|(from_book, _)| nearby_manuals.contains(&from_book))
    } else {
        match &recipe.autolearn {
            AutoLearn::Bool(autolearn) => {
                *autolearn
                    && recipe.skill_used.as_ref().map_or(true, |skill_used| {
                        recipe.difficulty
                            <= skills
                                .get(skill_used)
                                .unwrap_or_else(|| {
                                    panic!("Skill {:?} not found", recipe.skill_used)
                                })
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

fn qualities_present(required: &[RequiredQuality], present: &HashMap<&ObjectId, i8>) -> bool {
    required.iter().all(|required_quality| {
        present
            .get(&required_quality.id)
            .is_some_and(|present_level| required_quality.level as i8 <= *present_level)
    })
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_crafting_keyboard_input(
    mut keys: Keys,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut crafting_screen: ResMut<CraftingScreen>,
) {
    for combo in keys.combos(Ctrl::Without) {
        match combo.key {
            Key::Code(KeyCode::Escape) | Key::Character('&')
                if combo.change == InputChange::JustPressed =>
            {
                next_gameplay_state.set(GameplayScreenState::Base);
            }
            _ => {}
        }
    }
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
