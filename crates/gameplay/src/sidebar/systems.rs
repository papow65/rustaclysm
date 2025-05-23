use crate::DebugText;
use crate::sidebar::components::{
    BreathText, DetailsText, EnemiesText, FpsText, HealthText, LogDisplay, PlayerActionStateText,
    SpeedTextSpan, StaminaText, TimeText, WalkingModeTextSpan, WieldedText,
};
use crate::{
    Accessible, Actor, Amount, BaseSpeed, Breath, Clock, Corpse, CurrentlyVisibleBuilder,
    DebugTextShown, Envir, Explored, Faction, FocusState, Fragment, GameplayLocal, Health, Hurdle,
    Item, ItemHandler, ItemHierarchy, ItemItem, LastSeen, Life, Message, ObjectName, Obstacle,
    Opaque, OpaqueFloor, Phrase, Player, PlayerActionState, PlayerWielded, Pos, Positioning,
    RefreshAfterBehavior, RelativeSegments, SeenFrom, Severity, Shared, StairsDown, StairsUp,
    Stamina, StandardIntegrity, Timeouts, Visible, WalkingMode, ZoneLevelIds,
};
use application_state::ApplicationState;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::{hierarchy::Children, schedule::ScheduleConfigs, system::ScheduleSystem};
use bevy::picking::Pickable;
use bevy::prelude::{
    AlignItems, Changed, ChildOf, Commands, ComputedNode, Condition as _, DetectChanges as _,
    Entity, EntityCommands, EventReader, FlexDirection, FlexWrap, IntoScheduleConfigs as _,
    JustifyContent, Local, Node, Or, Overflow, ParamSet, PositionType, Query, Res, ScrollPosition,
    Single, SpawnRelated as _, State, StateScoped, Text, TextColor, TextSpan, UiRect, Val,
    Visibility, With, Without, children, info, on_event, resource_exists,
    resource_exists_and_changed, warn,
};
use cdda_json_files::{CharacterInfo, MoveCost};
use hud::{
    BAD_TEXT_COLOR, Fonts, HARD_TEXT_COLOR, PANEL_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
    panel_node, text_color_expect_half,
};
use std::{num::Saturating, time::Instant};
use util::log_if_slow;

type DuplicateMessageCount = Saturating<u16>;

const TEXT_WIDTH: f32 = 8.0 * 43.0; // 43 chars

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_sidebar(mut commands: Commands, fonts: Res<Fonts>) {
    let mut parent = commands.spawn((
        Node {
            top: Val::Px(0.0),
            right: Val::Px(0.0),
            width: Val::Px(TEXT_WIDTH + 10.0), // 5px margin on both sides
            height: Val::Percent(100.0),
            ..panel_node()
        },
        PANEL_COLOR,
        StateScoped(ApplicationState::Gameplay),
        Pickable::IGNORE,
    ));

    spawn_status_display(&fonts, &mut parent);
    spawn_log_display(&fonts, &mut parent);
}

fn spawn_status_display(fonts: &Fonts, parent: &mut EntityCommands) {
    parent.with_children(|child_builder| {
        child_builder
            .spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Px(TEXT_WIDTH),
                height: Val::Percent(100.0),
                margin: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..Node::default()
            })
            .with_children(|parent| {
                parent.spawn((Text::default(), SOFT_TEXT_COLOR, fonts.regular(), FpsText));
                parent.spawn((Text::default(), SOFT_TEXT_COLOR, fonts.regular(), TimeText));
                parent
                    .spawn((
                        Text::default(),
                        SOFT_TEXT_COLOR,
                        fonts.regular(),
                        HealthText,
                    ))
                    .with_children(|parent| {
                        parent.spawn((TextSpan::new("% health"), SOFT_TEXT_COLOR, fonts.regular()));
                    });
                parent
                    .spawn((
                        Text::default(),
                        SOFT_TEXT_COLOR,
                        fonts.regular(),
                        StaminaText,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextSpan::new("% stamina"),
                            SOFT_TEXT_COLOR,
                            fonts.regular(),
                        ));
                    });
                parent
                    .spawn((
                        Text::default(),
                        SOFT_TEXT_COLOR,
                        fonts.regular(),
                        BreathText,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextSpan::default(),
                            SOFT_TEXT_COLOR,
                            fonts.regular(),
                            WalkingModeTextSpan,
                        ));
                        parent.spawn((TextSpan::new(" ("), SOFT_TEXT_COLOR, fonts.regular()));
                        parent.spawn((
                            TextSpan::default(),
                            SOFT_TEXT_COLOR,
                            fonts.regular(),
                            SpeedTextSpan,
                        ));
                        parent.spawn((TextSpan::new(" km/h)"), SOFT_TEXT_COLOR, fonts.regular()));
                    });
                parent.spawn((
                    Text::default(),
                    SOFT_TEXT_COLOR,
                    fonts.regular(),
                    PlayerActionStateText,
                ));
                parent.spawn((
                    Text::new("Weapon: "),
                    SOFT_TEXT_COLOR,
                    fonts.regular(),
                    WieldedText,
                ));
                parent.spawn((
                    Text::default(),
                    SOFT_TEXT_COLOR,
                    fonts.regular(),
                    EnemiesText,
                ));
                parent.spawn((
                    Text::default(),
                    SOFT_TEXT_COLOR,
                    fonts.regular(),
                    DetailsText,
                ));
            });
    });
}

fn spawn_log_display(fonts: &Fonts, parent: &mut EntityCommands) {
    // TODO properly use flex layout
    parent.with_children(|child_builder| {
        child_builder.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Px(TEXT_WIDTH),
                height: Val::Px(20.0 * 16.0),
                margin: UiRect::all(Val::Px(5.0)),
                ..Node::default()
            },
            Pickable::IGNORE,
            //bevy::prelude::BackgroundColor(bevy::prelude::Srgba::RED.with_alpha(0.4).into()),
            children![(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    overflow: Overflow::scroll_y(),
                    // align_items: AlignItems::End, // TODO This looks better, but it breaks scrolling.
                    ..Node::default()
                },
                Pickable::default(),
                //bevy::prelude::BackgroundColor(bevy::prelude::Srgba::GREEN.with_alpha(0.4).into()),
                children![(
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..Node::default()
                    },
                    Pickable::IGNORE,
                    //bevy::prelude::BackgroundColor(bevy::prelude::Srgba::RED.with_alpha(0.4).into()),
                    children![(
                        Text::default(),
                        fonts.regular(),
                        Node {
                            flex_wrap: FlexWrap::Wrap,
                            ..Node::default()
                        },
                        //bevy::prelude::BackgroundColor(bevy::prelude::Srgba::BLUE.with_alpha(0.4).into()),
                        Pickable::IGNORE,
                        LogDisplay,
                    )]
                ),],
            ),],
        ));
    });
}

pub(super) fn update_sidebar_systems() -> ScheduleConfigs<ScheduleSystem> {
    (
        // sidebar components, in order:
        // (fps is handled elsewhere)
        update_status_time.run_if(resource_exists_and_changed::<Timeouts>),
        update_status_health.run_if(resource_exists_and_changed::<Timeouts>),
        update_status_stamina.run_if(resource_exists_and_changed::<Timeouts>),
        update_status_speed.run_if(on_event::<RefreshAfterBehavior>),
        update_status_player_action_state
            .run_if(resource_exists_and_changed::<State<PlayerActionState>>),
        update_status_player_wielded.run_if(resource_exists_and_changed::<Timeouts>),
        update_status_enemies.run_if(
            resource_exists_and_changed::<Timeouts>.and(resource_exists::<RelativeSegments>),
        ),
        update_status_detais.run_if(
            resource_exists_and_changed::<State<PlayerActionState>>
                .or(resource_exists_and_changed::<State<FocusState>>),
        ),
        update_log.run_if(on_event::<Message>),
        manage_log_wrapper,
    )
        .into_configs()
}

#[expect(clippy::needless_pass_by_value)]
fn update_log(
    mut commands: Commands,
    mut new_messages: EventReader<Message>,
    currently_visible_builder: CurrentlyVisibleBuilder,
    fonts: Res<Fonts>,
    debug_text_shown: Res<DebugTextShown>,
    log: Single<Entity, With<LogDisplay>>,
    mut previous_sections: GameplayLocal<Vec<(TextSpan, TextColor, Option<DebugText>)>>,
    mut last_message: GameplayLocal<Option<(Message, DuplicateMessageCount)>>,
    mut transient_message: Local<Vec<(TextSpan, TextColor, Option<DebugText>)>>,
) {
    const SINGLE: DuplicateMessageCount = Saturating(1);

    let start = Instant::now();

    for message in new_messages.read() {
        let percieved_message = percieve(&currently_visible_builder, message.clone());
        if message.phrase.to_string().trim() != "" {
            let suffix = if percieved_message.is_some() {
                ""
            } else {
                " (not perceived)"
            };
            if message.severity == Severity::Error {
                warn!("{}{suffix}", &message.phrase);
            } else {
                info!("{}{suffix}", &message.phrase);
            }
        }
        if let Some(message) = percieved_message {
            transient_message.clear();
            if message.transient {
                transient_message.extend(to_text_sections(&message, SINGLE));
            } else {
                match last_message.get() {
                    &mut Some((ref last, ref mut count)) if *last == message => {
                        *count += 1;
                    }
                    _ => {
                        if let Some((previous_last, previous_count)) =
                            last_message.get().replace((message, SINGLE))
                        {
                            previous_sections
                                .get()
                                .extend(to_text_sections(&previous_last, previous_count));
                        }
                    }
                }
            }
        }
    }

    let debug_font = debug_text_shown.text_font(fonts.regular());
    let mut log = commands.entity(*log);
    log.despawn_related::<Children>();
    log.with_children(|parent| {
        for (span, color, debug) in previous_sections.get().clone() {
            let mut entity = parent.spawn((span, color, fonts.regular(), Pickable::IGNORE));
            if let Some(debug) = debug {
                entity.insert((debug, debug_font.clone()));
            }
        }
        if let Some((message, count)) = last_message.get() {
            for (span, color, debug) in to_text_sections(message, *count) {
                let mut entity = parent.spawn((span, color, fonts.regular(), Pickable::IGNORE));
                if let Some(debug) = debug {
                    entity.insert((debug, debug_font.clone()));
                }
            }
        }
        for (span, color, debug) in transient_message.clone() {
            let mut entity = parent.spawn((span, color, fonts.regular(), Pickable::IGNORE));
            if let Some(debug) = debug {
                entity.insert((debug, debug_font.clone()));
            }
        }
    });

    log_if_slow("update_log", start);
}

/// Adapt the log wrapper to the log after its layout has been updated
fn manage_log_wrapper(
    mut commands: Commands,
    log: Option<Single<(&ComputedNode, &ChildOf), (With<LogDisplay>, Changed<ComputedNode>)>>,
    parents: Query<&ChildOf>,
) {
    let Some(log) = log else {
        return;
    };

    let (computed_node, log_parent) = *log;
    let log_height = computed_node.content_size.y;

    let log_wrapper = log_parent.parent();
    commands.entity(log_wrapper).insert((Node {
        width: Val::Percent(100.0),
        height: Val::Px(log_height),
        align_items: AlignItems::End,
        ..Node::default()
    },));

    let log_scroll = parents
        .get(log_wrapper)
        .expect("Log wrapper should have a parent")
        .parent();
    commands.entity(log_scroll).insert((ScrollPosition {
        offset_x: 0.0,
        offset_y: log_height,
    },));
}

fn to_text_sections(
    message: &Message,
    count: DuplicateMessageCount,
) -> Vec<(TextSpan, TextColor, Option<DebugText>)> {
    let mut sections = message
        .phrase
        .clone()
        .color_override(message.severity.color_override())
        .as_text_sections();
    if 1 < count.0 {
        sections.push((TextSpan::new(format!(" ({count}x)")), SOFT_TEXT_COLOR, None));
    }
    sections.push((TextSpan::from("\n"), SOFT_TEXT_COLOR, None));
    sections
}

fn percieve(
    currently_visible_builder: &CurrentlyVisibleBuilder,
    mut message: Message,
) -> Option<Message> {
    let mut seen = false;
    let mut global = true;

    for fragment in &mut message.phrase.fragments {
        match fragment.positioning {
            Positioning::Pos(pos) => {
                if currently_visible_builder
                    .for_player(true)
                    .can_see(pos, None)
                    == Visible::Seen
                {
                    seen = true;
                } else {
                    fragment.text = String::from("(unseen)");
                }
                global = false;
            }
            Positioning::Player => {
                seen = true;
                global = false;
            }
            Positioning::None => {
                // nothing to do
            }
        }
    }

    (seen || global).then_some(message)
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_status_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut text: Single<&mut Text, With<FpsText>>,
) {
    let start = Instant::now();

    if diagnostics.is_changed() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps) = fps.smoothed() {
                // Precision of 0.1s
                // Padding to 6 characters, aligned right
                text.0 = format!("{fps:05.1} fps\n");
            }
        }
    }

    log_if_slow("update_status_fps", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_time(clock: Clock, mut text: Single<&mut Text, With<TimeText>>) {
    let start = Instant::now();

    let now = clock.time();
    let sunlight = 100.0 * clock.sunlight_percentage();
    text.0 = format!("{now} ({sunlight:.0}% sunlight)\n\n");

    log_if_slow("update_status_time", start);
}

fn update_status_health(
    health: Option<Single<&Health, (With<Player>, Changed<Health>)>>,
    text: Single<(&mut Text, &mut TextColor), With<HealthText>>,
) {
    let start = Instant::now();

    if let Some(health) = health {
        let (mut text, mut color) = text.into_inner();

        text.0 = format!("{:3}", health.0.current());
        *color = health.0.color();

        //trace!("{:?}", ((health, text, style));
    }

    log_if_slow("update_status_health", start);
}

fn update_status_stamina(
    player_staminas: Option<Single<&Stamina, (With<Player>, Changed<Stamina>)>>,
    text: Single<(&mut Text, &mut TextColor), With<StaminaText>>,
) {
    let start = Instant::now();

    if let Some(player_stamina) = player_staminas {
        let Stamina::Limited(player_stamina) = *player_stamina else {
            panic!("{:?}", *player_stamina);
        };

        let (mut text, mut color) = text.into_inner();

        text.0 = format!("{:3.0}", 100.0 * player_stamina.relative());
        *color = player_stamina.color();

        //trace!("{:?}", ((player_stamina, text, style));
    }

    log_if_slow("update_status_stamina", start);
}

fn update_status_speed(
    player_actor: Option<
        Single<
            Actor,
            (
                With<Player>,
                Or<(Changed<BaseSpeed>, Changed<Stamina>, Changed<WalkingMode>)>,
            ),
        >,
    >,
    mut text_parts: ParamSet<(
        Single<(&mut Text, &mut TextColor), With<BreathText>>,
        Single<(&mut TextSpan, &mut TextColor), With<WalkingModeTextSpan>>,
        Single<(&mut TextSpan, &mut TextColor), With<SpeedTextSpan>>,
    )>,
) {
    let start = Instant::now();

    let Some(player_actor) = player_actor else {
        return;
    };

    let walking_mode = player_actor.walking_mode;

    let breath_text = text_parts.p0();
    let (mut text, mut color) = breath_text.into_inner();
    (text.0, *color) = match player_actor.stamina.breath() {
        Breath::Normal => (String::new(), HARD_TEXT_COLOR),
        Breath::AlmostWinded => (String::from("Almost winded "), WARN_TEXT_COLOR),
        Breath::Winded => (String::from("Winded "), BAD_TEXT_COLOR),
    };

    let walking_mode_text_span = text_parts.p1();
    let (mut text_span, mut color) = walking_mode_text_span.into_inner();
    text_span.0 = String::from(walking_mode.as_str());
    *color = walking_mode.breath_color();

    let speed_text_span = text_parts.p2();
    let (mut text_span, mut color) = speed_text_span.into_inner();
    let kmph = player_actor.speed().as_kmph();
    text_span.0 = if kmph < 9.95 {
        format!("{kmph:.1}")
    } else {
        format!("{kmph:.0}")
    };
    *color = text_color_expect_half(kmph / 15.0);

    log_if_slow("update_status_speed", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_player_action_state(
    player_action_state: Res<State<PlayerActionState>>,
    text: Single<(&mut Text, &mut TextColor), With<PlayerActionStateText>>,
) {
    let start = Instant::now();

    let (mut text, mut color) = text.into_inner();
    text.0 = format!("{}\n", **player_action_state);
    *color = player_action_state.color_in_progress();

    log_if_slow("update_status_player_action_state", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_player_wielded(
    mut commands: Commands,
    fonts: Res<Fonts>,
    debug_text_shown: Res<DebugTextShown>,
    player_weapon: Option<Single<Item, With<PlayerWielded>>>,
    text: Single<Entity, With<WieldedText>>,
) {
    let start = Instant::now();

    commands
        .entity(*text)
        .despawn_related::<Children>()
        .with_children(|parent| {
            if let Some(weapon) = player_weapon {
                let phrase = Phrase::from_fragments(weapon.fragments().collect());
                for (span, color, debug) in phrase.as_text_sections() {
                    let mut entity = parent.spawn((span, color, fonts.regular()));
                    if let Some(debug) = debug {
                        entity.insert((debug, debug_text_shown.text_font(fonts.regular())));
                    }
                }
            } else {
                parent.spawn((TextSpan::new("(none)"), SOFT_TEXT_COLOR, fonts.regular()));
            }
        });

    log_if_slow("update_status_wielded", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_enemies(
    mut commands: Commands,
    fonts: Res<Fonts>,
    debug_text_shown: Res<DebugTextShown>,
    currently_visible_builder: CurrentlyVisibleBuilder,
    player_actor: Single<Actor, With<Player>>,
    factions: Query<(&Pos, &Faction), With<Life>>,
    text: Single<Entity, With<EnemiesText>>,
) {
    let start = Instant::now();

    let factions = factions.iter().map(|(p, f)| (*p, f)).collect::<Vec<_>>();
    let mut enemies = Faction::Human.enemies(&currently_visible_builder, &factions, &player_actor);
    enemies.sort_by_key(|&pos| pos.vision_distance(*player_actor.pos).as_tiles());

    let phrase = Phrase::new("Enemies:")
        .extend(if enemies.is_empty() {
            vec![Fragment::soft("(none)")]
        } else {
            enemies
                .iter()
                .map(|&pos| {
                    (
                        pos,
                        currently_visible_builder
                            .envir
                            .find_character(pos)
                            .expect("Enemy should be present"),
                    )
                })
                .map(|(pos, (_, name))| {
                    Phrase::from_fragment(name.single(pos))
                        .hard((pos - *player_actor.pos).player_hint())
                        .fragments
                })
                .collect::<Vec<_>>()
                .join(&Fragment::soft(","))
        })
        .soft("\n");

    commands
        .entity(*text)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for (span, color, debug) in phrase.as_text_sections() {
                let mut entity = parent.spawn((span, color, fonts.regular()));
                if let Some(debug) = debug {
                    entity.insert((debug, debug_text_shown.text_font(fonts.regular())));
                }
            }
        });

    log_if_slow("update_status_enemies", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_detais(
    mut commands: Commands,
    focus_state: Res<State<FocusState>>,
    fonts: Res<Fonts>,
    debug_text_shown: Res<DebugTextShown>,
    explored: Res<Explored>,
    zone_level_ids: Res<ZoneLevelIds>,
    envir: Envir,
    item_hierarchy: ItemHierarchy,
    characters: Query<(
        &Shared<CharacterInfo>,
        &ObjectName,
        &Health,
        Option<&StandardIntegrity>,
    )>,
    entities: Query<
        (
            Option<&ObjectName>,
            Option<&Corpse>,
            Option<&Accessible>,
            Option<&StairsUp>,
            Option<&StairsDown>,
            Option<&StandardIntegrity>,
            Option<&Obstacle>,
            Option<&Hurdle>,
            Option<&Opaque>,
            Option<&OpaqueFloor>,
            Option<&LastSeen>,
            Option<&Visibility>,
        ),
        (Without<Health>, Without<Amount>),
    >,
    items: Query<Item>,
    text: Single<Entity, With<DetailsText>>,
) {
    let start = Instant::now();

    let text_sections = Phrase::from_fragments(match **focus_state {
        FocusState::Normal => vec![Fragment::soft(" ")], // Fragment added as a Bevy 0.15-dev workaround
        FocusState::ExaminingPos(pos) => {
            let mut total = vec![Fragment::soft(format!("\n{pos:?}\n"))];
            if explored.has_pos_been_seen(pos) {
                total.extend(characters_info(
                    envir.location.all(pos).copied(),
                    &characters,
                    pos,
                ));
                total.extend(
                    envir
                        .location
                        .all(pos)
                        .flat_map(|e| entities.get(*e))
                        .flat_map(entity_info),
                );
                total.extend({
                    let items = envir.location.all(pos).flat_map(|e| items.get(*e));
                    let mut handler = SidebarItemHandler { output: Vec::new() };
                    item_hierarchy.walk(&mut handler, items);
                    handler.output
                });
            } else {
                total.push(Fragment::soft("Unseen"));
            }
            total
        }
        FocusState::ExaminingZoneLevel(zone_level) => {
            vec![Fragment::soft(
                match explored.has_zone_level_been_seen(zone_level) {
                    seen_from @ Some(SeenFrom::CloseBy | SeenFrom::FarAway) => format!(
                        "\n{zone_level:?}\n{:?}\n{seen_from:?}",
                        zone_level_ids.get(zone_level)
                    ),
                    None | Some(SeenFrom::Never) => format!("\n{zone_level:?}\nUnseen"),
                },
            )]
        }
    })
    .as_text_sections();

    commands
        .entity(*text)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for (span, color, debug) in text_sections {
                let mut entity = parent.spawn((span, color, fonts.regular()));
                if let Some(debug) = debug {
                    entity.insert((debug, debug_text_shown.text_font(fonts.regular())));
                }
            }
        });

    log_if_slow("update_status_detais", start);
}

fn characters_info(
    all_here: impl Iterator<Item = Entity>,
    characters: &Query<(
        &Shared<CharacterInfo>,
        &ObjectName,
        &Health,
        Option<&StandardIntegrity>,
    )>,
    pos: Pos,
) -> Vec<Fragment> {
    all_here
        .flat_map(|i| characters.get(i))
        .flat_map(|(info, name, health, integrity)| {
            let start = Phrase::from_fragment(name.single(pos)).soft("(");

            if health.0.is_nonzero() {
                start
                    .push(Fragment::colorized(
                        format!("{}", health.0.current()),
                        health.0.color(),
                    ))
                    .push(Fragment::colorized("health", health.0.color()))
            } else {
                match integrity {
                    Some(integrity) if integrity.0.is_max() => {
                        start.push(Fragment::filthy("fresh"))
                    }
                    Some(integrity) => start
                        .push(Fragment::colorized(
                            format!("{:.0}", 100.0 - 100.0 * integrity.0.relative()),
                            integrity.0.color(),
                        ))
                        .push(Fragment::warn("% pulped")),
                    None => start.push(Fragment::good("thoroughly pulped")),
                }
            }
            .soft(")\n- ")
            .hard(&*info.id.fallback_name())
            .soft("\n")
            .fragments
        })
        .collect()
}

fn entity_info(
    (
        name,
        corpse,
        accessible,
        stairs_up,
        stairs_down,
        integrity,
        obstacle,
        hurdle,
        opaque,
        opaque_floor,
        last_seen,
        visibility,
    ): (
        Option<&ObjectName>,
        Option<&Corpse>,
        Option<&Accessible>,
        Option<&StairsUp>,
        Option<&StairsDown>,
        Option<&StandardIntegrity>,
        Option<&Obstacle>,
        Option<&Hurdle>,
        Option<&Opaque>,
        Option<&OpaqueFloor>,
        Option<&LastSeen>,
        Option<&Visibility>,
    ),
) -> Vec<Fragment> {
    let mut flags = Vec::new();
    if corpse.is_some() {
        flags.push("corpse");
    }
    let accessible_str: String;
    if let Some(accessible) = accessible {
        flags.push("accessible");
        if MoveCost::default() < accessible.move_cost {
            let factor =
                f32::from(accessible.move_cost.value()) / f32::from(MoveCost::default().value());
            accessible_str = format!("hurlde (x{factor:.1})");
            flags.push(accessible_str.as_str());
        }
    }
    if stairs_up.is_some() {
        flags.push("stairs up");
    }
    if stairs_down.is_some() {
        flags.push("stairs down");
    }
    let integrity_str;
    if let Some(integrity) = integrity {
        integrity_str = format!("integrity ({})", integrity.0.current());
        flags.push(integrity_str.as_str());
    }
    if obstacle.is_some() {
        flags.push("obstacle");
    }
    let hurdle_str;
    if let Some(hurdle) = hurdle {
        hurdle_str = format!("hurdle ({})", hurdle.0.0);
        flags.push(hurdle_str.as_str());
    }
    if opaque.is_some() {
        flags.push("opaque");
    }
    if opaque_floor.is_some() {
        flags.push("opaque_floor");
    }
    if let Some(last_seen) = last_seen {
        match *last_seen {
            LastSeen::Currently => flags.push("currently seen"),
            LastSeen::Previously => flags.push("previously seen"),
            LastSeen::Never => flags.push("never seen"),
        }
    }
    if let Some(visibility) = visibility {
        if visibility == Visibility::Hidden {
            flags.push("invisible");
        } else {
            flags.push("visible");
        }
    }

    let fallbak_name = ObjectName::missing();
    let mut output = Phrase::from_fragment(name.unwrap_or(&fallbak_name).single(Pos::ORIGIN));
    output = output.soft("\n");
    for flag in &flags {
        output = output.soft("- ").hard(*flag).soft("\n");
    }
    output.fragments
}

struct SidebarItemHandler {
    output: Vec<Fragment>,
}

impl ItemHandler for SidebarItemHandler {
    fn handle_item(&mut self, _item: &ItemItem, item_fragments: Vec<Fragment>) {
        self.output.extend(item_fragments);
    }

    fn show_other_pockets(&self) -> bool {
        true
    }
}
