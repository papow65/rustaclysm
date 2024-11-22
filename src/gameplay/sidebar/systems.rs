use crate::gameplay::sidebar::components::{
    BreathText, DetailsText, EnemiesText, FpsText, HealthText, LogDisplay, PlayerActionStateText,
    SpeedTextSpan, StaminaText, TimeText, WalkingModeTextSpan, WieldedText,
};
use crate::gameplay::{DebugText, InPocket, Subitems};
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
    FlexDirection, FlexWrap, IntoSystemConfigs, JustifyContent, Local, Node, Or, Overflow,
    ParamSet, PositionType, Query, Res, ResMut, State, StateScoped, Text, TextColor, TextSpan,
    UiRect, Val, Visibility, With, Without,
};
use cdda_json_files::{MoveCost, ObjectId, PocketType};
use std::iter::once;
use std::{num::Saturating, time::Instant};

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
    debug_text_shown: Res<DebugTextShown>,
    logs: Query<Entity, With<LogDisplay>>,
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
                eprintln!("{}{suffix}", &message.phrase);
            } else {
                println!("{}{suffix}", &message.phrase);
            }
        }
        if let Some(message) = percieved_message {
            transient_message.clear();
            if message.transient {
                transient_message.extend(to_text_sections(&message, SINGLE));
            } else {
                match last_message.get() {
                    Some((last, ref mut count)) if *last == message => {
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
    let mut logs = commands.entity(logs.single());
    logs.despawn_descendants();
    logs.with_children(|parent| {
        for (span, color, debug) in previous_sections.get().clone() {
            let mut entity = parent.spawn((span, color, fonts.regular()));
            if let Some(debug) = debug {
                entity.insert((debug, debug_font.clone()));
            }
        }
        if let Some((message, count)) = last_message.get() {
            for (span, color, debug) in to_text_sections(message, *count) {
                let mut entity = parent.spawn((span, color, fonts.regular()));
                if let Some(debug) = debug {
                    entity.insert((debug, debug_font.clone()));
                }
            }
        }
        for (span, color, debug) in transient_message.clone() {
            let mut entity = parent.spawn((span, color, fonts.regular()));
            if let Some(debug) = debug {
                entity.insert((debug, debug_font.clone()));
            }
        }
    });

    log_if_slow("update_log", start);
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
    debug_text_shown: Res<DebugTextShown>,
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

        let entity = text.single();
        commands
            .entity(entity)
            .despawn_descendants()
            .with_children(|parent| {
                for (span, color, debug) in phrase.as_text_sections() {
                    let mut entity = parent.spawn((span, color, fonts.regular()));
                    if let Some(debug) = debug {
                        entity.insert((debug, debug_text_shown.text_font(fonts.regular())));
                    }
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
    infos: Res<Infos>,
    debug_text_shown: Res<DebugTextShown>,
    mut explored: ResMut<Explored>,
    mut zone_level_ids: ResMut<ZoneLevelIds>,
    mut overmap_buffer_manager: OvermapBufferManager,
    mut overmap_manager: OvermapManager,
    envir: Envir,
    item_hierarchy: ItemHierarchy,
    characters: Query<(
        &ObjectDefinition,
        &ObjectName,
        &Health,
        Option<&StandardIntegrity>,
    )>,
    entities: Query<
        (
            Option<&ObjectDefinition>,
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
    text: Query<Entity, With<DetailsText>>,
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
                total.extend(
                    envir
                        .location
                        .all(pos)
                        .flat_map(|e| items.get(*e))
                        .flat_map(|item| item_info(&infos, &item_hierarchy, &item)),
                );
            } else {
                total.push(Fragment::soft("Unseen"));
            }
            total
        }
        FocusState::ExaminingZoneLevel(zone_level) => {
            vec![Fragment::soft(
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
    .as_text_sections();

    let entity = text.single();
    commands
        .entity(entity)
        .despawn_descendants()
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
        &ObjectDefinition,
        &ObjectName,
        &Health,
        Option<&StandardIntegrity>,
    )>,
    pos: Pos,
) -> Vec<Fragment> {
    all_here
        .flat_map(|i| characters.get(i))
        .flat_map(|(definition, name, health, integrity)| {
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
            .soft(")\n- ")
            .hard(&*definition.id.fallback_name())
            .soft("\n")
            .fragments
        })
        .collect()
}

fn entity_info(
    (
        definition,
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
        Option<&ObjectDefinition>,
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
    let mut output = Phrase::from_fragment(name.unwrap_or(&fallbak_name).single(Pos::ORIGIN));
    if let Some(definition) = definition {
        output = output.debug(format!(
            "[{:?}:{}]",
            definition.category,
            definition.id.fallback_name()
        ));
    }
    output = output.soft("\n");
    for flag in &flags {
        output = output.soft("- ").hard(*flag).soft("\n");
    }
    output.fragments
}

fn item_info(infos: &Infos, item_hierarchy: &ItemHierarchy, item: &ItemItem) -> Vec<Fragment> {
    item_hierarchy.walk(&SidebarItemWalker { infos }, None, item.entity)
}

struct SidebarItemWalker<'i> {
    infos: &'i Infos,
}

impl<'i> SidebarItemWalker<'i> {
    fn prefix(in_pocket: Option<InPocket>) -> String {
        let Some(in_pocket) = in_pocket else {
            return String::new();
        };
        let indicator = match in_pocket.type_ {
            PocketType::Container => {
                if in_pocket.single_in_type {
                    return String::new();
                } else {
                    '>'
                }
            }
            PocketType::Magazine => {
                return String::from("with");
            }
            PocketType::MagazineWell => {
                return String::from("(");
            }
            PocketType::Mod => '+',
            PocketType::Corpse => '_',
            PocketType::Software => 'S',
            PocketType::Ebook => 'E',
            PocketType::Migration => 'M',
            PocketType::Last => '9',
        };
        format!("{}'-{indicator}", "    ".repeat(in_pocket.depth.get() - 1))
    }

    const fn suffix(in_pocket: Option<InPocket>) -> &'static str {
        match in_pocket {
            Some(
                InPocket {
                    type_: PocketType::Magazine,
                    ..
                }
                | InPocket {
                    type_: PocketType::Container,
                    single_in_type: true,
                    ..
                },
            ) => "",
            Some(InPocket {
                type_: PocketType::MagazineWell,
                ..
            }) => ")",
            _ => "\n",
        }
    }
}

impl<'i> ItemHierarchyWalker for SidebarItemWalker<'i> {
    fn visit_item<'p>(
        &'p self,
        item: ItemItem,
        contents: impl Iterator<Item = Subitems<'p>>,
        magazines: impl Iterator<Item = Subitems<'p>>,
        magazine_wells: impl Iterator<Item = Subitems<'p>>,
        other_pockets: impl Iterator<Item = Subitems<'p>>,
        in_pocket: Option<InPocket>,
    ) -> Vec<Fragment> {
        let prefix = Self::prefix(in_pocket);
        let suffix = Self::suffix(in_pocket);

        let contents = contents.collect::<Vec<_>>();
        let is_container = 0 < contents.iter().len();
        let is_empty = contents.iter().all(|info| info.output.is_empty());
        let is_sealed = contents.iter().all(|info| info.pocket.sealed);
        let direct_subitems = contents.iter().map(|info| info.direct_items).sum::<usize>();

        // TODO make sure all pockets are present on containers
        //println!("{:?} {is_container:?} {is_empty:?}", item.definition.id.fallback_name());

        let mut magazine_output = magazines
            .flat_map(|subitems| subitems.output)
            .collect::<Vec<_>>();

        let phrase = Phrase::from_fragment(Fragment::soft(prefix.clone()))
            .extend({
                self.infos
                    .try_magazine(&item.definition.id)
                    .filter(|magazine| {
                        magazine.ammo_type.as_ref().is_some_and(|ammo_type| {
                            ammo_type.0.contains(&ObjectId::new("battery"))
                        })
                    })
                    .map(|magazine| {
                        #[allow(clippy::iter_with_drain)] // don't drop 'magazine_output'
                        magazine_output.drain(..).chain(once(Fragment::soft(
                            magazine
                                .capacity
                                .map_or_else(String::new, |capacity| format!("/{capacity}")),
                        )))
                    })
                    .into_iter()
                    .flatten()
            })
            .extend(item.fragments())
            .debug(format!("[{}]", item.definition.id.fallback_name()))
            .extend(magazine_output)
            .extend(magazine_wells.flat_map(|info| {
                if info.output.is_empty() {
                    vec![Fragment::colorized("not loaded", SOFT_TEXT_COLOR)]
                } else {
                    info.output
                }
            }))
            .soft(match (is_container, is_empty, is_sealed) {
                (true, true, true) => "(empty, sealed)",
                (true, true, false) => "(empty)",
                (true, false, true) => "(sealed)",
                _ => "",
            });

        if !is_container || is_empty {
            phrase.soft(suffix)
        } else if direct_subitems == 1 {
            phrase
                .push(Fragment::colorized(">", GOOD_TEXT_COLOR))
                .extend(contents.into_iter().flat_map(|info| info.output))
                .soft(suffix)
        } else {
            phrase
                .push(Fragment::colorized(
                    format!("> {direct_subitems}+"),
                    GOOD_TEXT_COLOR,
                ))
                .soft(suffix)
                .extend(contents.into_iter().flat_map(|info| info.output))
        }
        .extend(other_pockets.flat_map(|info| {
            Some(info.output)
                .filter(|output| !output.is_empty())
                .map(|output| {
                    once(Fragment::soft(format!(
                        "{prefix}{:?}:\n",
                        info.pocket.type_
                    )))
                    .chain(output)
                })
                .into_iter()
                .flatten()
        }))
        .fragments
    }
}
