use crate::screens::{find_nearby, find_nearby_pseudo, nearby_tools};
use crate::{BodyContainers, GameplayScreenState, Item, Player, Shared};
use bevy::platform::collections::HashSet;
use bevy::prelude::{
    AnyOf, Commands, DespawnOnExit, Entity, EntityCommands, In, KeyCode, Local, NextState,
    Pickable, Query, Res, ResMut, Single, SpawnRelated as _, Text, TextColor, TextSpan, With,
    World, children,
};
use cdda_json_files::{CommonItemInfo, FurnitureInfo, InfoId, TerrainInfo, UseAction};
use gameplay_location::{LocationCache, Pos};
use gameplay_model::LastSeen;
use hud::{
    Fonts, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR, scroll_screen,
};
use keyboard::KeyBindings;
use manual::ManualSection;
use selection_list::SelectionList;
use std::collections::{BTreeMap, btree_map::Entry};
use std::{iter::once, sync::Arc, time::Instant};
use util::{log_if_slow, uppercase_first};

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_tool_screen(
    mut commands: Commands,
    location: Res<LocationCache>,
    fonts: Res<Fonts>,
    player: Single<(&Pos, &BodyContainers), With<Player>>,
    items: Query<(Item, &LastSeen)>,
    infrastructure: Query<(
        AnyOf<(&Shared<FurnitureInfo>, &Shared<TerrainInfo>)>,
        &LastSeen,
    )>,
) {
    let action_list = scroll_screen(&mut commands, GameplayScreenState::Tool);

    add_actions(
        &location,
        &fonts,
        *player,
        &items,
        &infrastructure,
        &mut commands.entity(action_list),
    );
}

fn add_actions(
    location: &LocationCache,
    fonts: &Fonts,
    (&player_pos, body_containers): (&Pos, &BodyContainers),
    items: &Query<(Item, &LastSeen)>,
    infrastructure: &Query<(
        AnyOf<(&Shared<FurnitureInfo>, &Shared<TerrainInfo>)>,
        &LastSeen,
    )>,
    action_list: &mut EntityCommands,
) {
    let nearby_items = find_nearby(location, items, player_pos, body_containers);
    //info!("{:?}", &nearby_items);
    let nearby_pseudo_items = find_nearby_pseudo(location, infrastructure, player_pos);
    //info!("{:?}", &nearby_pseudo_items);
    let nearby_tools_ = nearby_tools(&nearby_items, &nearby_pseudo_items);
    let nearby_tool_actions = tool_actions(nearby_tools_);

    let mut selection_list = SelectionList::default();

    action_list
        .with_children(|parent| {
            parent.spawn((
                Text::from("Actions using nearby tools:"),
                WARN_TEXT_COLOR,
                fonts.regular(),
                Pickable::IGNORE,
            ));

            for ((action, _, level), items) in nearby_tool_actions {
                let action = uppercase_first(action);
                let items = items
                    .into_iter()
                    .map(|(_, name)| name)
                    .collect::<Vec<_>>()
                    .join(", ");
                let child_entity = parent
                    .spawn((
                        Text::from(&*action),
                        HARD_TEXT_COLOR,
                        fonts.regular(),
                        Pickable::IGNORE,
                        children![
                            (
                                TextSpan::from(if let Some(level) = level {
                                    format!(" ({level})")
                                } else {
                                    String::new()
                                }),
                                SOFT_TEXT_COLOR,
                                fonts.regular(),
                                Pickable::IGNORE,
                            ),
                            (
                                TextSpan::from(": "),
                                SOFT_TEXT_COLOR,
                                fonts.regular(),
                                Pickable::IGNORE,
                            ),
                            (
                                TextSpan::from(items),
                                HARD_TEXT_COLOR,
                                fonts.regular(),
                                Pickable::IGNORE,
                            )
                        ],
                    ))
                    .id();
                selection_list.append(child_entity);
            }
        })
        .insert(selection_list);
}

fn tool_actions(
    nearby_tools: impl Iterator<Item = Arc<CommonItemInfo>>,
) -> BTreeMap<(Arc<str>, Option<Arc<str>>, Option<u8>), HashSet<(InfoId<CommonItemInfo>, Arc<str>)>>
{
    nearby_tools
        .flat_map(|item_info| {
            let use_actions = item_info.use_action.0.iter().filter_map(|use_action| {
                use_action.id().get_option().map(|item_action_id| {
                    (
                        item_action_id,
                        matches!(&use_action, UseAction::Typed(..))
                            .then_some(item_info.name.single.clone()),
                        use_action.level(),
                        item_info.id.clone(),
                        item_info.name.single.clone(),
                    )
                })
            });

            let quality_actions = item_info
                .qualities
                .iter()
                .filter_map(|quality| quality.id.get_option())
                .flat_map(|quality| quality.usages.clone())
                .flat_map(|(level, item_actions)| {
                    item_actions
                        .iter()
                        .map(|item_action| (item_action.clone(), level))
                        .collect::<Vec<_>>()
                })
                .filter_map(|(item_action, level)| {
                    item_action
                        .get_option()
                        .map(|item_action| (item_action, level))
                })
                .map(|(item_action, level)| {
                    (
                        item_action,
                        None,
                        Some(level),
                        item_info.id.clone(),
                        item_info.name.single.clone(),
                    )
                });

            use_actions.chain(quality_actions).collect::<Vec<_>>()
        })
        .fold(
            BTreeMap::<
                (Arc<str>, Option<Arc<str>>, Option<u8>),
                HashSet<(InfoId<CommonItemInfo>, Arc<str>)>,
            >::new(),
            |mut acc, (action, unique, level, id, name)| {
                match acc.entry((action.name.str.clone(), unique, level)) {
                    Entry::Vacant(vacant) => {
                        vacant.insert(once((id, name)).collect());
                    }
                    Entry::Occupied(mut occupied) => {
                        occupied.get_mut().insert((id, name));
                    }
                }
                acc
            },
        )
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn create_tool_key_bindings(
    world: &mut World,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    fresh_bindings.spawn(world, GameplayScreenState::Tool, |bindings| {
        bindings.add(KeyCode::Escape, exit_tools);
        bindings.add('t', exit_tools);
    });

    world.spawn((
        ManualSection::new(&[("close tools", "esc/t")], 100),
        DespawnOnExit(GameplayScreenState::Tool),
    ));

    log_if_slow("create_tool_key_bindings", start);
}

fn exit_tools(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    let start = Instant::now();

    next_gameplay_state.set(GameplayScreenState::Base);

    log_if_slow("exit_tools", start);
}

pub(super) fn adapt_to_tool_selection(
    In((previous_selected, selected)): In<(Option<Entity>, Option<Entity>)>,
    mut text_colors: Query<&mut TextColor>,
) {
    let start = Instant::now();

    if let Some(previous_selected) = previous_selected {
        let text_color = &mut text_colors
            .get_mut(previous_selected)
            .expect("Previous highlighted tool action should be found");
        **text_color = HARD_TEXT_COLOR;
    }

    if let Some(selected) = selected {
        let text_color = &mut text_colors
            .get_mut(selected)
            .expect("Highlighted tool action should be found");
        **text_color = GOOD_TEXT_COLOR;
    }

    log_if_slow("adapt_to_tool_selection", start);
}
