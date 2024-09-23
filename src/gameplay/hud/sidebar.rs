use crate::common::{
    log_if_slow, on_safe_event, Fonts, ScrollingList, BAD_TEXT_COLOR, DEFAULT_TEXT_COLOR,
    FILTHY_COLOR, GOOD_TEXT_COLOR, WARN_TEXT_COLOR,
};
use crate::gameplay::hud::components::{LogDisplay, StatusDisplay};
use crate::gameplay::hud::resources::{HudDefaults, StatusTextSections};
use crate::{application::ApplicationState, gameplay::*};
use ::cdda::{MoveCost, ObjectId};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{
    resource_exists, resource_exists_and_changed, BuildChildren, Changed, Commands, Condition,
    DetectChanges, Entity, EventReader, FlexDirection, FlexWrap, FromWorld, IntoSystemConfigs,
    JustifyContent, Local, NodeBundle, Or, Overflow, Parent, PositionType, Query, Res, ResMut,
    State, StateScoped, Style, Text, TextBundle, TextSection, TextStyle, UiRect, Val, Visibility,
    With, Without, World,
};
use std::time::Instant;
use units::{Mass, Volume};

const TEXT_WIDTH: f32 = 8.0 * 43.0; // 43 chars

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_sidebar(
    mut commands: Commands,
    hud_defaults: Res<HudDefaults>,
    mut text_sections: ResMut<StatusTextSections>,
) {
    text_sections.fps.style = hud_defaults.text_style.clone();
    text_sections.time.style = hud_defaults.text_style.clone();
    text_sections.player_action_state.style = hud_defaults.text_style.clone();
    for section in &mut text_sections.health {
        section.style = hud_defaults.text_style.clone();
    }
    for section in &mut text_sections.stamina {
        section.style = hud_defaults.text_style.clone();
    }
    for section in &mut text_sections.speed {
        section.style = hud_defaults.text_style.clone();
    }

    let mut background = hud_defaults.background.clone();
    background.style.top = Val::Px(0.0);
    background.style.right = Val::Px(0.0);
    background.style.width = Val::Px(TEXT_WIDTH + 10.0); // 5px margin on both sides
    background.style.height = Val::Percent(100.0);
    let mut parent = commands.spawn((background, StateScoped(ApplicationState::Gameplay)));
    spawn_status_display(&mut parent);
    spawn_log_display(&mut parent);
}

fn spawn_status_display(parent: &mut EntityCommands) {
    // TODO properly use flex layout
    parent.with_children(|child_builder| {
        child_builder.spawn((
            TextBundle {
                text: Text::default(),
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Px(TEXT_WIDTH),
                    height: Val::Percent(100.0),
                    margin: UiRect::all(Val::Px(5.0)),
                    ..Style::default()
                },
                ..TextBundle::default()
            },
            StatusDisplay,
        ));
    });
}

fn spawn_log_display(parent: &mut EntityCommands) {
    // TODO properly use flex layout

    parent.with_children(|child_builder| {
        child_builder
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Px(TEXT_WIDTH),
                    height: Val::Px(20.0 * 16.0),
                    margin: UiRect::all(Val::Px(5.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::End,
                    overflow: Overflow::clip(),
                    ..Style::default()
                },
                ..NodeBundle::default()
            })
            .with_children(|child_builder| {
                child_builder.spawn((
                    TextBundle {
                        text: Text {
                            sections: vec![],
                            ..Text::default()
                        },
                        style: Style {
                            width: Val::Px(TEXT_WIDTH),
                            flex_wrap: FlexWrap::Wrap,
                            ..Style::default()
                        },
                        ..TextBundle::default()
                    },
                    ScrollingList::default(),
                    LogDisplay,
                ));
            });
    });
}

pub(super) fn update_sidebar_systems() -> impl IntoSystemConfigs<()> {
    (
        // sidebar components, in order:
        // (fps is handled elsewhere)
        update_status_time.run_if(resource_exists_and_changed::<Timeouts>),
        update_status_health.run_if(resource_exists_and_changed::<Timeouts>),
        update_status_stamina.run_if(resource_exists_and_changed::<Timeouts>),
        update_status_speed.run_if(on_safe_event::<RefreshAfterBehavior>()),
        update_status_player_action_state
            .run_if(resource_exists_and_changed::<State<PlayerActionState>>),
        update_status_player_wielded.run_if(resource_exists_and_changed::<Timeouts>),
        update_status_enemies.run_if(resource_exists_and_changed::<Timeouts>),
        update_status_detais.run_if(
            resource_exists_and_changed::<State<PlayerActionState>>
                .or_else(resource_exists_and_changed::<State<FocusState>>),
        ),
        update_log.run_if(on_safe_event::<Message>()),
    )
        .chain()
        .run_if(resource_exists::<StatusTextSections>.and_then(resource_exists::<RelativeSegments>))
}

#[expect(clippy::needless_pass_by_value)]
fn update_log(
    mut new_messages: EventReader<Message>,
    currently_visible_builder: CurrentlyVisibleBuilder,
    hud_defaults: Res<HudDefaults>,
    mut session: GameplaySession,
    mut logs: Query<&mut Text, With<LogDisplay>>,
    mut previous_sections: Local<Vec<TextSection>>,
    mut last_message: Local<Option<(Message, usize)>>,
    mut transient_message: Local<Vec<TextSection>>,
) {
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
                transient_message.extend(to_text_sections(&hud_defaults.text_style, &message, 1));
            } else {
                match &mut *last_message {
                    Some((last, ref mut count)) if *last == message => {
                        *count += 1;
                    }
                    _ => {
                        if let Some((previous_last, previous_count)) =
                            last_message.replace((message, 1))
                        {
                            previous_sections.extend(to_text_sections(
                                &hud_defaults.text_style,
                                &previous_last,
                                previous_count,
                            ));
                        }
                    }
                }
            }
        }
    }

    let sections = &mut logs
        .iter_mut()
        .next()
        .expect("Exactly one log text")
        .sections;
    sections.clear();
    sections.extend(previous_sections.clone());
    if let Some((message, count)) = &*last_message {
        sections.extend(to_text_sections(&hud_defaults.text_style, message, *count));
    }
    sections.extend(transient_message.clone());

    log_if_slow("update_log", start);
}

fn to_text_sections(text_style: &TextStyle, message: &Message, count: usize) -> Vec<TextSection> {
    let mut style = text_style.clone();
    style.color = message.severity.color();
    let mut sections = message.phrase.as_text_sections(&style);
    if 1 < count {
        sections.push(TextSection {
            value: format!(" ({count}x)"),
            style: text_style.clone(),
        });
    }
    sections.push(TextSection {
        value: String::from("\n"),
        style: text_style.clone(),
    });
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

fn update_status_display(text_sections: &StatusTextSections, status_display: &mut Text) {
    status_display.sections = vec![text_sections.fps.clone(), text_sections.time.clone()];
    status_display.sections.extend(text_sections.health.clone());
    status_display
        .sections
        .extend(text_sections.stamina.clone());
    status_display.sections.extend(text_sections.speed.clone());
    status_display
        .sections
        .push(text_sections.player_action_state.clone());
    status_display
        .sections
        .extend(text_sections.wielded.clone());
    status_display
        .sections
        .extend(text_sections.enemies.clone());
    status_display
        .sections
        .extend(text_sections.details.clone());
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_status_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if diagnostics.is_changed() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Precision of 0.1s
                // Padding to 6 characters, aligned right
                text_sections.fps.value = format!("{average:05.1} fps\n");
            }
        }

        update_status_display(&text_sections, &mut status_displays.single_mut());
    }

    log_if_slow("update_status_fps", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_time(
    clock: Clock,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    let now = clock.time();
    let sunlight = 100.0 * clock.sunlight_percentage();
    text_sections.time.value = format!("{now} ({sunlight:.0}% sunlight)\n\n");
    update_status_display(&text_sections, &mut status_displays.single_mut());

    log_if_slow("update_status_time", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_health(
    health: Query<&Health, (With<Player>, Changed<Health>)>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Ok(health) = health.get_single() {
        text_sections.health[0].value = format!("{}", health.0.current());
        text_sections.health[0].style.color = health.0.color();

        text_sections.health[1].value = String::from(" health\n");

        update_status_display(&text_sections, &mut status_displays.single_mut());
    }

    log_if_slow("update_status_health", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_stamina(
    player_staminas: Query<&Stamina, (With<Player>, Changed<Stamina>)>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Ok(Stamina::Limited(player_stamina)) = player_staminas.get_single() {
        text_sections.stamina[0].value = format!("{}", player_stamina.current());
        text_sections.stamina[0].style.color = player_stamina.color();

        text_sections.stamina[1].value = String::from(" stamina\n");

        update_status_display(&text_sections, &mut status_displays.single_mut());
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
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Ok(player_actor) = player_actors.get_single() {
        let walking_mode = player_actor.walking_mode;
        (
            text_sections.speed[0].value,
            text_sections.speed[0].style.color,
        ) = match player_actor.stamina.breath() {
            Breath::Normal => (String::new(), DEFAULT_TEXT_COLOR),
            Breath::AlmostWinded => (String::from("Almost winded "), WARN_TEXT_COLOR),
            Breath::Winded => (String::from("Winded "), BAD_TEXT_COLOR),
        };

        text_sections.speed[1].value = String::from(walking_mode.as_str());
        text_sections.speed[1].style.color = walking_mode.color();

        text_sections.speed[2].value = format!(" ({})\n", player_actor.speed());
        text_sections.speed[2].style.color = text_sections.speed[0].style.color;

        update_status_display(&text_sections, &mut status_displays.single_mut());
    }

    log_if_slow("update_status_speed", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_player_action_state(
    player_action_state: Res<State<PlayerActionState>>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    text_sections.player_action_state.value = format!("{}\n", **player_action_state);
    text_sections.player_action_state.style.color = player_action_state.color_in_progress();

    update_status_display(&text_sections, &mut status_displays.single_mut());

    log_if_slow("update_status_player_action_state", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_player_wielded(
    fonts: Res<Fonts>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
    player_weapon: Query<Item, With<PlayerWielded>>,
) {
    let start = Instant::now();

    let begin = Phrase::new("Weapon:");
    let phrase = if let Ok(weapon) = player_weapon.get_single() {
        begin.extend(weapon.fragments())
    } else {
        begin.add("(none)")
    }
    .add("\n");

    let text_style = HudDefaults::new(&fonts).text_style;
    text_sections.wielded = phrase.as_text_sections(&text_style);

    update_status_display(&text_sections, &mut status_displays.single_mut());

    log_if_slow("update_status_wielded", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_enemies(
    currently_visible_builder: CurrentlyVisibleBuilder,
    player_actors: Query<Actor, With<Player>>,
    factions: Query<(&Pos, &Faction), With<Life>>,
    fonts: Res<Fonts>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
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

        let text_style = HudDefaults::new(&fonts).text_style;
        text_sections.enemies = phrase.as_text_sections(&text_style);

        update_status_display(&text_sections, &mut status_displays.single_mut());
    }

    log_if_slow("update_status_enemies", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_status_detais(
    focus_state: Res<State<FocusState>>,
    hud_defaults: Res<HudDefaults>,
    mut explored: ResMut<Explored>,
    mut zone_level_ids: ResMut<ZoneLevelIds>,
    mut text_sections: ResMut<StatusTextSections>,
    mut overmap_buffer_manager: OvermapBufferManager,
    envir: Envir,
    mut overmap_manager: OvermapManager,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
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
) {
    let start = Instant::now();

    text_sections.details = Phrase::from_fragments(match **focus_state {
        FocusState::Normal => Vec::new(),
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
    .as_text_sections(&hud_defaults.text_style);

    update_status_display(&text_sections, &mut status_displays.single_mut());

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
            .add(definition.id.fallback_name())
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
