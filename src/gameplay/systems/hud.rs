use crate::prelude::*;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::{EntityCommands, Resource};
use bevy::prelude::*;
use std::time::Instant;
use time::OffsetDateTime;

const TEXT_WIDTH: f32 = 8.0 * 43.0; // 43 chars

#[derive(Resource)]
pub(crate) struct HudDefaults {
    text_style: TextStyle,
}

impl HudDefaults {
    pub(crate) fn new(font: Handle<Font>) -> Self {
        Self {
            text_style: TextStyle {
                font,
                font_size: 16.0,
                color: SOFT_TEXT_COLOR,
            },
        }
    }
}

fn spawn_log_display(parent: &mut EntityCommands) {
    // TODO properly use flex layout

    parent.with_children(|child_builder| {
        child_builder
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(0.0),
                        left: Val::Px(0.0),
                        ..UiRect::default()
                    },
                    size: Size {
                        width: Val::Px(TEXT_WIDTH),
                        height: Val::Px(20.0 * 16.0),
                    },
                    margin: UiRect::all(Val::Px(5.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::End,
                    overflow: Overflow::Hidden,
                    ..default()
                },
                ..default()
            })
            .with_children(|child_builder| {
                child_builder
                    .spawn(TextBundle {
                        text: Text {
                            sections: vec![],
                            ..Text::default()
                        },
                        style: Style {
                            size: Size::width(Val::Px(TEXT_WIDTH)),
                            flex_wrap: FlexWrap::Wrap,
                            ..Style::default()
                        },
                        ..TextBundle::default()
                    })
                    .insert(LogDisplay);
            });
    });
}

fn spawn_status_display(parent: &mut EntityCommands) {
    // TODO properly use flex layout
    parent.with_children(|child_builder| {
        child_builder
            .spawn(TextBundle {
                text: Text::default(),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        ..UiRect::default()
                    },
                    size: Size {
                        width: Val::Px(TEXT_WIDTH),
                        height: Val::Percent(100.0),
                    },
                    margin: UiRect::all(Val::Px(5.0)),
                    ..Style::default()
                },
                ..TextBundle::default()
            })
            .insert(StatusDisplay);
    });
}

fn spawn_manual_display(
    commands: &mut Commands,
    hud_defaults: &HudDefaults,
    mut background: NodeBundle,
) {
    background.style.position.bottom = Val::Px(0.0);
    background.style.position.left = Val::Px(0.0);

    commands
        .spawn(background)
        .insert(ManualDisplay)
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: String::new()
                                + "move          numpad\n"
                                + "up/down       </>\n"
                                + "wield         w\n"
                                + "pick/drop     b/v\n"
                                + "attack        a\n"
                                + "smash         s\n"
                                + "toggle speed  +\n"
                                + "wait          |\n"
                                + "sleep         $\n"
                                + "examine       x\n"
                                + "examine map   m\n"
                                + "zoom          (shift+)z\n"
                                + "zoom          scroll wheel\n"
                                + "show elevated h\n"
                                + "toggle this   f1\n"
                                + "menu          esc\n"
                                + "main menu     f12\n"
                                + "quit          ctrl+c/d/q",
                            style: hud_defaults.text_style.clone(),
                        }],
                        ..Text::default()
                    },
                    ..default()
                })
                .insert(ManualDisplay);
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_hud(
    mut commands: Commands,
    fonts: Res<Fonts>,
    mut text_sections: ResMut<StatusTextSections>,
) {
    let hud_defaults = HudDefaults::new(fonts.default());

    text_sections.fps.style = hud_defaults.text_style.clone();
    text_sections.time.style = hud_defaults.text_style.clone();
    text_sections.state.style = hud_defaults.text_style.clone();
    for section in &mut text_sections.health {
        section.style = hud_defaults.text_style.clone();
    }
    for section in &mut text_sections.stamina {
        section.style = hud_defaults.text_style.clone();
    }
    for section in &mut text_sections.speed {
        section.style = hud_defaults.text_style.clone();
    }

    let mut background = NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        background_color: PANEL_COLOR.into(),
        ..default()
    };

    spawn_manual_display(&mut commands, &hud_defaults, background.clone());

    background.style.position.top = Val::Px(0.0);
    background.style.position.right = Val::Px(0.0);
    background.style.size = Size {
        width: Val::Px(TEXT_WIDTH + 10.0), // 5px margin on both sides
        height: Val::Percent(100.0),
    };
    let mut parent = commands.spawn(background);
    spawn_status_display(&mut parent);
    spawn_log_display(&mut parent);

    commands.insert_resource(hud_defaults);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_log(
    hud_defaults: Res<HudDefaults>,
    mut logs: Query<&mut Text, With<LogDisplay>>,
    messages: Query<&Message>,
    changed: Query<&Message, Changed<Message>>,
) {
    let start = Instant::now();

    let mut new_messages = false;
    for message in changed.iter() {
        if message.to_string().trim() != "" {
            if message.severity == Severity::Error {
                eprintln!("{}", &message);
            } else {
                println!("{}", &message);
            }
        }
        new_messages = true;
    }
    if !new_messages {
        return;
    }

    let mut last: Option<(&Message, usize)> = None;
    let mut shown_reverse = Vec::<(&Message, usize)>::new();
    for message in messages.iter().collect::<Vec<&Message>>().iter().rev() {
        match last {
            Some((last_message, ref mut last_count))
                if last_message.fragments == message.fragments =>
            {
                *last_count += 1;
            }
            Some(message_and_count) => {
                shown_reverse.push(message_and_count);
                if 20 <= shown_reverse.len() {
                    last = None;
                    break;
                } else {
                    last = Some((message, 1));
                }
            }
            None => {
                last = Some((message, 1));
            }
        }
    }
    if let Some(message_and_count) = last {
        shown_reverse.push(message_and_count);
    }

    let sections = &mut logs.iter_mut().next().unwrap().sections;
    sections.clear();
    for (message, count) in shown_reverse.into_iter().rev() {
        let mut style = hud_defaults.text_style.clone();
        style.color = message.severity.color();
        let cloned_message = Message {
            fragments: message.fragments.clone(),
            severity: message.severity.clone(),
        };
        sections.extend(cloned_message.into_text_sections(&style));
        if 1 < count {
            sections.push(TextSection {
                value: format!(" ({count}x)"),
                style: hud_defaults.text_style.clone(),
            });
        }
        sections.push(TextSection {
            value: String::from("\n"),
            style: hud_defaults.text_style.clone(),
        });
    }

    log_if_slow("update_log", start);
}

fn update_status_display(text_sections: &StatusTextSections, status_display: &mut Text) {
    status_display.sections = vec![text_sections.fps.clone(), text_sections.time.clone()];
    status_display.sections.extend(text_sections.health.clone());
    status_display
        .sections
        .extend(text_sections.stamina.clone());
    status_display.sections.extend(text_sections.speed.clone());
    status_display.sections.push(text_sections.state.clone());
    status_display
        .sections
        .extend(text_sections.details.clone());
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_fps(
    diagnostics: Res<Diagnostics>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if diagnostics.is_changed() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_time(
    timeouts: Res<Timeouts>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    let season_length = 91; // TODO load from worldoptions.json

    let tenth_seconds = timeouts.time().0 / 100;
    let seconds = tenth_seconds / 10;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let seasons = days / season_length;
    let years = seasons / 4;

    text_sections.time.value = format!(
        "{:#04}-{}-{:#02} {:#02}:{:#02}:{:#02}.{}\n\n",
        years + OffsetDateTime::now_utc().year() as u64 + 1, // based on https://cataclysmdda.org/lore-background.html
        match seasons % 4 {
            0 => "Spring",
            1 => "Summer",
            2 => "Autumn",
            3 => "Winter",
            _ => panic!("Modulo error"),
        },
        days % season_length + 1, // 1-based
        hours % 24,
        minutes % 60,
        seconds % 60,
        tenth_seconds % 10
    );
    update_status_display(&text_sections, &mut status_displays.single_mut());

    log_if_slow("update_status_time", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_health(
    health: Query<&Health, (With<Player>, Changed<Health>)>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Some(health) = health.iter().next() {
        text_sections.health[0].value = format!("{}", health.0.current());
        text_sections.health[0].style.color = health.0.color();

        text_sections.health[1].value = String::from(" health\n");

        update_status_display(&text_sections, &mut status_displays.single_mut());
    }

    log_if_slow("update_status_health", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_stamina(
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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_speed(
    player_actors: Query<
        ActorTuple,
        (
            With<Player>,
            Or<(Changed<BaseSpeed>, Changed<Stamina>, Changed<WalkingMode>)>,
        ),
    >,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Ok(player_tuple) = player_actors.get_single() {
        let player_actor = Actor::from(player_tuple);
        let walking_mode = player_actor.walking_mode;
        text_sections.speed[0].value = match player_actor.stamina.breath() {
            Breath::Normal => String::new(),
            Breath::Winded => String::from("Winded "),
        };
        text_sections.speed[0].style.color = BAD_TEXT_COLOR;

        text_sections.speed[1].value = String::from(walking_mode.as_str());
        text_sections.speed[1].style.color = walking_mode.color();

        text_sections.speed[2].value = format!(" ({})\n", player_actor.speed());
        text_sections.speed[2].style.color = match player_actor.stamina.breath() {
            Breath::Normal => DEFAULT_TEXT_COLOR,
            Breath::Winded => BAD_TEXT_COLOR,
        };

        update_status_display(&text_sections, &mut status_displays.single_mut());
    }

    log_if_slow("update_status_speed", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_player_state(
    player_action_state: Res<PlayerActionState>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    text_sections.state.value = format!("{}\n", *player_action_state);
    text_sections.state.style.color = player_action_state.color();

    update_status_display(&text_sections, &mut status_displays.single_mut());

    log_if_slow("update_status_player_state", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_detais(
    player_action_state: Res<PlayerActionState>,
    envir: Envir,
    asset_server: Res<AssetServer>,
    hud_defaults: Res<HudDefaults>,
    mut explored: ResMut<Explored>,
    mut zone_level_ids: ResMut<ZoneLevelIds>,
    mut text_sections: ResMut<StatusTextSections>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
    characters: Query<(&ObjectDefinition, &ObjectName, &Health)>,
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

    text_sections.details = Message::info()
        .extend(match *player_action_state {
            PlayerActionState::ExaminingPos(pos) => {
                let mut total = vec![Fragment::new(format!("\n{pos:?}\n"), DEFAULT_TEXT_COLOR)];
                if explored.has_pos_been_seen(pos) {
                    let all_here = envir.location.all(pos);
                    total.extend(characters_info(&all_here, &characters));
                    total.extend(
                        all_here
                            .iter()
                            .flat_map(|&i| entities.get(i))
                            .flat_map(entity_info)
                            .collect::<Vec<_>>(),
                    );
                } else {
                    total.push(Fragment::new(String::from("Unseen"), DEFAULT_TEXT_COLOR));
                }
                total
            }
            PlayerActionState::ExaminingZoneLevel(zone_level) => {
                vec![Fragment::new(
                    match explored.has_zone_level_been_seen(&asset_server, zone_level) {
                        seen_from @ Some(SeenFrom::CloseBy | SeenFrom::FarAway) => format!(
                            "\n{zone_level:?}\n{:?}\n{seen_from:?}",
                            zone_level_ids.get(zone_level)
                        ),
                        None | Some(SeenFrom::Never) => format!("\n{zone_level:?}\nUnseen"),
                    },
                    DEFAULT_TEXT_COLOR,
                )]
            }
            _ => Vec::new(),
        })
        .into_text_sections(&hud_defaults.text_style);

    update_status_display(&text_sections, &mut status_displays.single_mut());

    log_if_slow("update_status_detais", start);
}

fn characters_info(
    all_here: &[Entity],
    characters: &Query<(&ObjectDefinition, &ObjectName, &Health)>,
) -> Vec<Fragment> {
    all_here
        .iter()
        .flat_map(|&i| characters.get(i))
        .flat_map(|(definition, name, health)| {
            Message::info()
                .push(name.single())
                .str("(")
                .push(Fragment::new(
                    format!("{}", health.0.current()),
                    health.0.color(),
                ))
                .str("health)\n- ")
                .add(definition.id.fallback_name())
                .str("\n")
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
    if let Some(definition) = definition {
        id_str = format!("{:?}", definition.id);
        flags.push(id_str.as_str());
    }
    if corpse.is_some() {
        flags.push("corpse");
    }
    let accessible_str: String;
    if let Some(accessible) = accessible {
        flags.push("accessible");
        if MoveCost::default() < accessible.move_cost {
            let factor = f32::from(accessible.move_cost.0) / f32::from(MoveCost::default().0);
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
    let mut output = Message::info().extend(name.unwrap_or(&fallbak_name).as_item(amount, filthy));
    for flag in &flags {
        output = output.add(format!("\n- {flag}"));
    }
    output.str("\n").fragments
}
