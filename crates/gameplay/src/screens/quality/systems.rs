use crate::screens::{Nearby, find_nearby, find_nearby_pseudo, nearby_qualities};
use crate::{BodyContainers, GameplayScreenState, LastSeen, Location, Player, Pos, Shared};
use bevy::prelude::{
    AlignItems, AnyOf, ChildOf, Commands, FlexDirection, JustifyContent, KeyCode, Local, NextState,
    Node, Overflow, Query, Res, ResMut, Single, StateScoped, Text, UiRect, Val, With, World,
};
use cdda_json_files::{FurnitureInfo, TerrainInfo};
use hud::{Fonts, GOOD_TEXT_COLOR, PANEL_COLOR, SMALL_SPACING, ScrollList, WARN_TEXT_COLOR};
use keyboard::KeyBindings;
use manual::{LargeNode, ManualSection};
use std::time::Instant;
use util::{log_if_slow, uppercase_first};

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_crafting_screen(
    mut commands: Commands,
    location: Res<Location>,
    fonts: Res<Fonts>,
    player: Single<(&Pos, &BodyContainers), With<Player>>,
    items: Query<(Nearby, &LastSeen, Option<&ChildOf>)>,
    infrastructure: Query<(
        AnyOf<(&Shared<FurnitureInfo>, &Shared<TerrainInfo>)>,
        &LastSeen,
    )>,
) {
    let qualitiy_list = commands
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
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            StateScoped(GameplayScreenState::Quality),
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
                        .add_child(qualitiy_list);
                });
        });

    let (&player_pos, body_containers) = *player;

    let nearby_items = find_nearby(&location, &items, player_pos, body_containers);
    let nearby_pseudo_items = find_nearby_pseudo(&location, &infrastructure, player_pos);
    let nearby_qualities = nearby_qualities(&nearby_items, &nearby_pseudo_items);
    //trace!("{:?}", &nearby_manuals);

    let mut shown_qualities = nearby_qualities
        .iter()
        .map(|(quality, amount)| (amount, uppercase_first(quality.name.single.clone())))
        .collect::<Vec<_>>();
    shown_qualities.sort_by_key(|(.., name)| name.clone());

    commands.entity(qualitiy_list).with_children(|parent| {
        parent.spawn((
            Text::from("Nearby qualities:"),
            WARN_TEXT_COLOR,
            fonts.regular(),
        ));

        for (amount, name) in shown_qualities {
            parent.spawn((
                Text::from(format!("{amount} {name}")),
                GOOD_TEXT_COLOR,
                fonts.regular(),
            ));
        }
    });
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn create_crafting_key_bindings(
    world: &mut World,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    fresh_bindings.spawn(world, GameplayScreenState::Quality, |bindings| {
        bindings.add(KeyCode::Escape, exit_qualities);
        bindings.add('q', exit_qualities);
    });

    world.spawn((
        ManualSection::new(&[("close qualities", "esc/&")], 100),
        StateScoped(GameplayScreenState::Quality),
    ));

    log_if_slow("create_crafting_key_bindings", start);
}

fn exit_qualities(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    let start = Instant::now();

    next_gameplay_state.set(GameplayScreenState::Base);

    log_if_slow("exit_qualities", start);
}
