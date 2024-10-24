use crate::gameplay::sidebar::components::{
    BreathText, DetailsText, EnemiesText, FpsText, HealthText, LogDisplay, PlayerActionStateText,
    SpeedTextSpan, StaminaText, TimeText, WalkingModeTextSpan, WieldedText,
};
use crate::hud::{
    panel_node, text_color_expect_half, Fonts, ScrollList, BAD_TEXT_COLOR, FILTHY_COLOR,
    GOOD_TEXT_COLOR, HARD_TEXT_COLOR, PANEL_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
};
use crate::util::log_if_slow;
use crate::{application::ApplicationState, gameplay::*};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::{schedule::SystemConfigs, system::EntityCommands};
use bevy::prelude::{
    on_event, resource_exists, resource_exists_and_changed, AlignItems, BuildChildren, Changed,
    ChildBuild, Commands, Condition, DespawnRecursiveExt, DetectChanges, Entity, EventReader,
    FlexDirection, FlexWrap, FromWorld, IntoSystemConfigs, JustifyContent, Local, Node, Or,
    Overflow, ParamSet, Parent, PositionType, Query, Res, ResMut, State, StateScoped, Text,
    TextColor, TextFont, TextSpan, UiRect, Val, Visibility, With, Without, World,
};
use cdda_json_files::{MoveCost, ObjectId};
use std::{num::Saturating, time::Instant};
use units::{Mass, Volume};

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
        child_builder
            .spawn(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Px(TEXT_WIDTH),
                height: Val::Px(20.0 * 16.0),
                margin: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                overflow: Overflow::clip(),
                ..Node::default()
            })
            .with_children(|child_builder| {
                child_builder.spawn((
                    Text::default(),
                    fonts.regular(),
                    Node {
                        width: Val::Px(TEXT_WIDTH),
                        flex_wrap: FlexWrap::Wrap,
                        ..Node::default()
                    },
                    ScrollList::default(),
                    LogDisplay,
                ));
            });
    });
}

pub(super) fn update_sidebar_systems() -> SystemConfigs {
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
    )
        .into_configs()
}

#[expect(clippy::needless_pass_by_value)]
fn update_log(
    mut commands: Commands,
    mut new_messages: EventReader<Message>,
    currently_visible_builder: CurrentlyVisibleBuilder,
    fonts: Res<Fonts>,
    mut session: GameplaySession,
    logs: Query<Entity, With<LogDisplay>>,
    mut previous_sections: Local<Vec<(TextSpan, TextColor, TextFont)>>,
    mut last_message: Local<Option<(Message, DuplicateMessageCount)>>,
    mut transient_message: Local<Vec<(TextSpan, TextColor, TextFont)>>,
) {
    const SINGLE: DuplicateMessageCount = Saturating(1);

    let start = Instant::now();

    if session.is_changed() {
        *previous_sections = Vec::new();
        *last_message = Option::None;
    }

    for message in new_messages.read() {
        let percieved_message = percieve(&currently_visible_builder, message.clone());
        if message.phrase.to_string().trim() != "" {
            let suffix = if percieved_message.is_some() {
                ""
            } else {
                " (not perceived)"
            };
            if message.severity == Severity::Error {
                eprintln!("{}{suffix}", &message.phrase);
            } else {
                println!("{}{suffix}", &message.phrase);
            }
        }
        if let Some(message) = percieved_message {
            transient_message.clear();
            if message.transient {
                transient_message.extend(to_text_sections(&fonts, &message, SINGLE));
            } else {
                match &mut *last_message {
                    Some((last, ref mut count)) if *last == message => {
                        *count += 1;
                    }
                    _ => {
                        if let Some((previous_last, previous_count)) =
                            last_message.replace((message, SINGLE))
                        {
                            previous_sections.extend(to_text_sections(
                                &fonts,
                                &previous_last,
                                previous_count,
                            ));
                        }
                    }
                }
            }
        }
    }

    let mut logs = commands.entity(logs.single());
    logs.despawn_descendants();
    logs.with_children(|parent| {
        for section in previous_sections.clone() {
            parent.spawn(section);
        }
        if let Some((message, count)) = &*last_message {
            for section in to_text_sections(&fonts, message, *count) {
                parent.spawn(section);
            }
        }
        for section in transient_message.clone() {
            parent.spawn(section);
        }
    });

    log_if_slow("update_log", start);
}

fn to_text_sections(
    fonts: &Fonts,
    message: &Message,
    count: DuplicateMessageCount,
) -> Vec<(TextSpan, TextColor, TextFont)> {
    let mut sections = message
        .phrase
        .as_text_sections(message.severity.color(), &fonts.regular());
    if 1 < count.0 {
        sections.push((
            TextSpan::new(format!(" ({count}x)")),
            SOFT_TEXT_COLOR,
            fonts.regular(),
        ));
    }
    sections.push((TextSpan::from("\n"), SOFT_TEXT_COLOR, fonts.regular()));
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
    mut text: Query<&mut Text, With<FpsText>>,
) {
    let start = Instant::now();

    if diagnostics.is_changed() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps) = fps.smoothed() {
                let mut text = text.single_mut();
                // Precision of 0.1s
                // Padding to 6 characters, aligned right
                text.0 = format!("{fps:05.1} fps\n");
            }
        }
    }

    log_if_slow("update_status_fps", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_time(clock: Clock, mut text: Query<&mut Text, With<TimeText>>) {
    let start = Instant::now();

    let mut text = text.single_mut();

    let now = clock.time();
    let sunlight = 100.0 * clock.sunlight_percentage();
    text.0 = format!("{now} ({sunlight:.0}% sunlight)\n\n");

    log_if_slow("update_status_time", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_health(
    health: Query<&Health, (With<Player>, Changed<Health>)>,
    mut text: Query<(&mut Text, &mut TextColor), With<HealthText>>,
) {
    let start = Instant::now();

    if let Ok(health) = health.get_single() {
        let (mut text, mut color) = text.single_mut();

        text.0 = format!("{:3}", health.0.current());
        *color = health.0.color();

        //dbg!((health, text, style));
    }

    log_if_slow("update_status_health", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_stamina(
    player_staminas: Query<&Stamina, (With<Player>, Changed<Stamina>)>,
    mut text: Query<(&mut Text, &mut TextColor), With<StaminaText>>,
) {
    let start = Instant::now();

    if let Ok(Stamina::Limited(player_stamina)) = player_staminas.get_single() {
        let (mut text, mut color) = text.single_mut();

        text.0 = format!("{:3.0}", 100.0 * player_stamina.relative());
        *color = player_stamina.color();

        //dbg!((player_stamina, text, style));
    }

    log_if_slow("update_status_stamina", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_speed(
    player_actors: Query<
        Actor,
        (
            With<Player>,
            Or<(Changed<BaseSpeed>, Changed<Stamina>, Changed<WalkingMode>)>,
        ),
    >,
    mut text_parts: ParamSet<(
        Query<(&mut Text, &mut TextColor), With<BreathText>>,
        Query<(&mut TextSpan, &mut TextColor), With<WalkingModeTextSpan>>,
        Query<(&mut TextSpan, &mut TextColor), With<SpeedTextSpan>>,
    )>,
) {
    let start = Instant::now();

    if let Ok(player_actor) = player_actors.get_single() {
        let walking_mode = player_actor.walking_mode;

        let mut breath_text = text_parts.p0();
        let (mut text, mut color) = breath_text.single_mut();
        (text.0, *color) = match player_actor.stamina.breath() {
            Breath::Normal => (String::new(), HARD_TEXT_COLOR),
            Breath::AlmostWinded => (String::from("Almost winded "), WARN_TEXT_COLOR),
            Breath::Winded => (String::from("Winded "), BAD_TEXT_COLOR),
        };

        let mut walking_mode_text_span = text_parts.p1();
        let (mut text_span, mut color) = walking_mode_text_span.single_mut();
        text_span.0 = String::from(walking_mode.as_str());
        *color = walking_mode.breath_color();

        let mut speed_text_span = text_parts.p2();
        let (mut text_span, mut color) = speed_text_span.single_mut();
        let kmph = player_actor.speed().as_kmph();
        text_span.0 = if kmph < 9.95 {
            format!("{kmph:.1}")
        } else {
            format!("{kmph:.0}")
        };
        *color = text_color_expect_half(kmph / 15.0);
    }

    log_if_slow("update_status_speed", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_player_action_state(
    player_action_state: Res<State<PlayerActionState>>,
    mut text: Query<(&mut Text, &mut TextColor), With<PlayerActionStateText>>,
) {
    let start = Instant::now();

    let (mut text, mut color) = text.single_mut();
    text.0 = format!("{}\n", **player_action_state);
    *color = player_action_state.color_in_progress();

    log_if_slow("update_status_player_action_state", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_player_wielded(
    mut commands: Commands,
    fonts: Res<Fonts>,
    player_weapon: Query<Item, With<PlayerWielded>>,
    text: Query<Entity, With<WieldedText>>,
) {
    let start = Instant::now();

    let entity = text.single();
    commands
        .entity(entity)
        .despawn_descendants()
        .with_children(|parent| {
            if let Ok(weapon) = player_weapon.get_single() {
                let phrase = Phrase::from_fragments(weapon.fragments());
                for section in phrase.as_text_sections(HARD_TEXT_COLOR, &fonts.regular()) {
                    parent.spawn(section);
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
    currently_visible_builder: CurrentlyVisibleBuilder,
    player_actors: Query<Actor, With<Player>>,
    factions: Query<(&Pos, &Faction), With<Life>>,
    text: Query<Entity, With<EnemiesText>>,
) {
    let start = Instant::now();

    let factions = factions.iter().map(|(p, f)| (*p, f)).collect::<Vec<_>>();
    if let Ok(player_actor) = player_actors.get_single() {
        let mut enemies =
            Faction::Human.enemies(&currently_visible_builder, &factions, &player_actor);
        enemies.sort_by_key(|&pos| pos.vision_distance(*player_actor.pos).as_tiles());

        let begin = Phrase::new("Enemies:");
        let phrase = if enemies.is_empty() {
            begin.add("(none)")
        } else {
            begin.extend(
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
                            .add((pos - *player_actor.pos).player_hint())
                            .fragments
                    })
                    .collect::<Vec<_>>()
                    .join(&Fragment::new(",")),
            )
        }
        .add("\n");

        let entity = text.single();
        commands
            .entity(entity)
            .despawn_descendants()
            .with_children(|parent| {
                for section in phrase.as_text_sections(SOFT_TEXT_COLOR, &fonts.regular()) {
                    parent.spawn(section);
                }
            });
    }

    log_if_slow("update_status_enemies", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_detais(
    mut commands: Commands,
    focus_state: Res<State<FocusState>>,
    fonts: Res<Fonts>,
    mut explored: ResMut<Explored>,
    mut zone_level_ids: ResMut<ZoneLevelIds>,
    mut overmap_buffer_manager: OvermapBufferManager,
    envir: Envir,
    mut overmap_manager: OvermapManager,
    characters: Query<(&ObjectDefinition, &ObjectName, &Health, Option<&Integrity>)>,
    entities: Query<
        (
            Option<&ObjectDefinition>,
            Option<&ObjectName>,
            Option<&Amount>,
            Option<&Filthy>,
            Option<&Corpse>,
            Option<&Accessible>,
            Option<&StairsUp>,
            Option<&StairsDown>,
            Option<&Integrity>,
            Option<&Obstacle>,
            Option<&Hurdle>,
            Option<&Opaque>,
            Option<&OpaqueFloor>,
            Option<&LastSeen>,
            Option<&Visibility>,
        ),
        Without<Health>,
    >,
    text: Query<Entity, With<DetailsText>>,
) {
    let start = Instant::now();

    let text_sections = Phrase::from_fragments(match **focus_state {
        FocusState::Normal => vec![Fragment::new(" ")], // Fragment added as a Bevy 0.15-dev workaround
        FocusState::ExaminingPos(pos) => {
            let mut total = vec![Fragment::new(format!("\n{pos:?}\n"))];
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
                        .flat_map(|i| entities.get(*i))
                        .flat_map(entity_info),
                );
            } else {
                total.push(Fragment::new(String::from("Unseen")));
            }
            total
        }
        FocusState::ExaminingZoneLevel(zone_level) => {
            vec![Fragment::new(
                match explored.has_zone_level_been_seen(&mut overmap_buffer_manager, zone_level) {
                    seen_from @ Some(SeenFrom::CloseBy | SeenFrom::FarAway) => format!(
                        "\n{zone_level:?}\n{:?}\n{seen_from:?}",
                        zone_level_ids.get(&mut overmap_manager, zone_level)
                    ),
                    None | Some(SeenFrom::Never) => format!("\n{zone_level:?}\nUnseen"),
                },
            )]
        }
    })
    .as_text_sections(SOFT_TEXT_COLOR, &fonts.regular());

    let entity = text.single();
    commands
        .entity(entity)
        .despawn_descendants()
        .with_children(|parent| {
            for section in text_sections {
                parent.spawn(section);
            }
        });

    log_if_slow("update_status_detais", start);
}

fn characters_info(
    all_here: impl Iterator<Item = Entity>,
    characters: &Query<(&ObjectDefinition, &ObjectName, &Health, Option<&Integrity>)>,
    pos: Pos,
) -> Vec<Fragment> {
    all_here
        .flat_map(|i| characters.get(i))
        .flat_map(|(definition, name, health, integrity)| {
            let start = Phrase::from_fragment(name.single(pos)).add("(");

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
                        start.push(Fragment::colorized("fresh", FILTHY_COLOR))
                    }
                    Some(integrity) => start
                        .push(Fragment::colorized(
                            format!("{:.0}", 100.0 - 100.0 * integrity.0.relative()),
                            integrity.0.color(),
                        ))
                        .push(Fragment::colorized("% pulped", WARN_TEXT_COLOR)),
                    None => start.push(Fragment::colorized("thoroughly pulped", GOOD_TEXT_COLOR)),
                }
            }
            .add(")\n- ")
            .add(String::from(&*definition.id.fallback_name()))
            .add("\n")
            .fragments
        })
        .collect()
}

fn entity_info(
    (
        definition,
        name,
        amount,
        filthy,
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
        Option<&ObjectDefinition>,
        Option<&ObjectName>,
        Option<&Amount>,
        Option<&Filthy>,
        Option<&Corpse>,
        Option<&Accessible>,
        Option<&StairsUp>,
        Option<&StairsDown>,
        Option<&Integrity>,
        Option<&Obstacle>,
        Option<&Hurdle>,
        Option<&Opaque>,
        Option<&OpaqueFloor>,
        Option<&LastSeen>,
        Option<&Visibility>,
    ),
) -> Vec<Fragment> {
    let mut flags = Vec::new();
    let id_str;
    let category_str;
    if let Some(definition) = definition {
        id_str = format!("{:?}", definition.id);
        flags.push(id_str.as_str());
        category_str = format!("{:?}", definition.category);
        flags.push(category_str.as_str());
    }
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
    };
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
        hurdle_str = format!("hurdle ({})", hurdle.0 .0);
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
    let item = ItemItem {
        entity: Entity::PLACEHOLDER,
        definition: &ObjectDefinition {
            category: ObjectCategory::Meta,
            id: ObjectId::new(""),
        },
        name: name.unwrap_or(&fallbak_name),
        pos: None,
        amount: amount.unwrap_or(&Amount::SINGLE),
        filthy,
        containable: &Containable {
            volume: Volume::default(),
            mass: Mass::ZERO,
        },
        parent: &Parent::from_world(&mut World::default()),
    };
    let mut output = Phrase::from_fragments(item.fragments());
    for flag in &flags {
        output = output.add(format!("\n- {flag}"));
    }
    output.add("\n").fragments
}
