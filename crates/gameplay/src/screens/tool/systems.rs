use crate::screens::{find_nearby, find_nearby_pseudo, nearby_tools};
use crate::{BodyContainers, GameplayScreenState, Item, Player, Shared};
use bevy::ecs::system::EntityCommands;
use bevy::picking::Pickable;
use bevy::platform::collections::HashSet;
use bevy::prelude::{
    AlignItems, AnyOf, Commands, DespawnOnExit, FlexDirection, JustifyContent, KeyCode, Local,
    NextState, Node, Overflow, Query, Res, ResMut, Single, SpawnRelated as _, Text, TextSpan,
    UiRect, Val, With, World, children,
};
use cdda_json_files::{CommonItemInfo, FurnitureInfo, InfoId, TerrainInfo, UseAction};
use gameplay_location::{LocationCache, Pos};
use gameplay_model::LastSeen;
use hud::{Fonts, HARD_TEXT_COLOR, PANEL_COLOR, SMALL_SPACING, SOFT_TEXT_COLOR, WARN_TEXT_COLOR};
use keyboard::KeyBindings;
use manual::{LargeNode, ManualSection};
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
    let action_list = commands
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
            DespawnOnExit(GameplayScreenState::Tool),
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
                .with_children(|builder| {
                    builder
                        .spawn((
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
                        ))
                        .add_child(action_list);
                });
        });

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
    //let nearby_tools = nearby_tools(&nearby_items, &nearby_pseudo_items);
    //let nearby_tool_quality_actions = tool_quality_actions(nearby_tools);

    action_list.with_children(|parent| {
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
            parent.spawn((
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
            ));
        }
    });
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
