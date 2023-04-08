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
            .spawn(TextBundle {
                text: Text {
                    sections: vec![],
                    ..Text::default()
                },
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
                    flex_wrap: FlexWrap::Wrap,
                    ..Style::default()
                },
                ..TextBundle::default()
            })
            .insert(LogDisplay);
    });
}

fn spawn_status_display(hud_defaults: &HudDefaults, parent: &mut EntityCommands) {
    // TODO properly use flex layout
    parent.with_children(|child_builder| {
        child_builder
            .spawn(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: String::from("\n"),
                            style: hud_defaults.text_style.clone(),
                        };
                        6
                    ],
                    ..Text::default()
                },
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
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: String::from("move          numpad\nup/down       r/f/</>\nwield         w\npick/drop     b/v\nattack        a\nsmash         s\nwait          |\nrun           +\nexamine       x\nexamine map   m\nzoom          (shift+)z\nzoom          scroll wheel\nshow elevated e\ntoggle this   h\nmenu          esc\nmain menu     f12\nquit          ctrl+c/d/q"),
                            style: hud_defaults.text_style.clone(),
                        },
                    ],
                    ..Text::default()
                },
                ..default()
            })
            .insert(ManualDisplay);
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_hud(mut commands: Commands, fonts: Res<Fonts>) {
    let hud_defaults = HudDefaults::new(fonts.default());

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
    spawn_status_display(&hud_defaults, &mut parent);
    spawn_log_display(&mut parent);

    commands.insert_resource(hud_defaults);

    // Workaround to make sure the log starts at the bottom
    for i in 0..20 {
        // All different to make sure the messages are not bundled
        commands.spawn(Message::info(" ".repeat(i)));
    }
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
        if message.text.trim() != "" {
            if message.severity == Severity::Error {
                eprintln!("{}", message.text);
            } else {
                println!("{}", message.text);
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
            Some((last_message, ref mut last_count)) if last_message.text == message.text => {
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
    for (message, count) in shown_reverse.iter().rev() {
        let mut style = hud_defaults.text_style.clone();
        style.color = message.severity.color();
        sections.push(TextSection {
            value: message.text.clone(),
            style,
        });
        if 1 < *count {
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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_fps(
    diagnostics: Res<Diagnostics>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if diagnostics.is_changed() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Precision of 0.1s
                // Padding to 6 characters, aligned right
                status_displays.iter_mut().next().unwrap().sections[0].value =
                    format!("{average:05.1} fps\n");
            }
        }
    }

    log_if_slow("update_status_fps", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_time(
    timeouts: Res<Timeouts>,
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

    status_displays.iter_mut().next().unwrap().sections[1].value = format!(
        "{:#04}-{}-{:#02} {:#02}:{:#02}:{:#02}.{}\n",
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

    log_if_slow("update_status_time", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_health(
    health: Query<&Health, (With<Player>, Changed<Health>)>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Some(health) = health.iter().next() {
        let text_section = &mut status_displays.iter_mut().next().unwrap().sections[2];
        text_section.value = format!("\n{} health\n", health.0.current());
        text_section.style.color = health.0.color();
    }

    log_if_slow("update_status_health", start);
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
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Ok(actor_tuple) = player_actors.get_single() {
        status_displays.iter_mut().next().unwrap().sections[3].value =
            format!("{}\n", Actor::from(actor_tuple).speed());
    }

    log_if_slow("update_status_speed", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_stamina(
    player_staminas: Query<&Stamina, (With<Player>, Changed<Stamina>)>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Ok(player_stamina) = player_staminas.get_single() {
        let text_section = &mut status_displays.iter_mut().next().unwrap().sections[4];
        text_section.value = format!("{} stamina\n", player_stamina.0.current());
        text_section.style.color = player_stamina.0.color();
    }

    log_if_slow("update_status_speed", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_player_state(
    player_action_state: Res<PlayerActionState>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    status_displays.iter_mut().next().unwrap().sections[5].value =
        format!("{}\n", *player_action_state);

    log_if_slow("update_status_player_state", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_detais(
    player_action_state: Res<PlayerActionState>,
    envir: Envir,
    asset_server: Res<AssetServer>,
    mut explored: ResMut<Explored>,
    mut zone_level_ids: ResMut<ZoneLevelIds>,
    characters: Query<(Option<&ObjectDefinition>, Option<&TextLabel>, &Health)>,
    entities: Query<
        (
            Option<&ObjectDefinition>,
            Option<&TextLabel>,
            Option<&Corpse>,
            Option<&Accessible>,
            Option<&StairsUp>,
            Option<&StairsDown>,
            Option<&Obstacle>,
            Option<&Hurdle>,
            Option<&Opaque>,
            Option<&OpaqueFloor>,
            Option<&LastSeen>,
            Option<&Visibility>,
        ),
        (Without<Health>, Without<Amount>),
    >,
    items: Query<(Option<&ObjectDefinition>, &TextLabel), (Without<Health>, With<Amount>)>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
    //_globs: Query<(&GlobalTransform, Option<&ZoneLevel>)>,
) {
    let start = Instant::now();

    status_displays.iter_mut().next().unwrap().sections[5].value = match *player_action_state {
        PlayerActionState::ExaminingPos(pos) => {
            /*for ent in envir.location.all(pos) {
                if let Ok((glob, _)) = globs.get(ent) {
                    println!("Global transfrom: {glob:?}");
                }
            }*/

            if explored.has_pos_been_seen(pos) {
                let all_here = envir.location.all(pos);
                let characters = characters_info(&all_here, &characters);
                let entities = all_here
                    .iter()
                    .flat_map(|&i| entities.get(i))
                    .map(entity_info)
                    .map(|s| s + "\n")
                    .collect::<String>();
                let items = items_info(&all_here, &items);
                format!("\n{pos:?}\n{characters}{entities}{items}")
            } else {
                format!("\n{pos:?}\nUnseen")
            }
        }
        PlayerActionState::ExaminingZoneLevel(zone_level) => {
            /*for (glob, z) in globs.iter() {
                if z == Some(&zone_level) {
                    println!("Global transfrom: {glob:?}");
                }
            }*/

            match explored.has_zone_level_been_seen(&asset_server, zone_level) {
                seen_from @ Some(SeenFrom::CloseBy | SeenFrom::FarAway) => format!(
                    "\n{zone_level:?}\n{:?}\n{seen_from:?}",
                    zone_level_ids.get(zone_level)
                ),
                None | Some(SeenFrom::Never) => format!("\n{zone_level:?}\nUnseen"),
            }
        }
        _ => String::new(),
    };

    log_if_slow("update_status_detais", start);
}

fn characters_info(
    all_here: &[Entity],
    characters: &Query<(Option<&ObjectDefinition>, Option<&TextLabel>, &Health)>,
) -> String {
    all_here
        .iter()
        .flat_map(|&i| characters.get(i))
        .map(|(_definition, label, health)| {
            let label = label.map_or_else(|| String::from("[Unknown]"), String::from);
            format!("{label} ({} health)\n", health.0.current())
        })
        .collect()
}

fn entity_info(
    (
        definition,
        label,
        corpse,
        accessible,
        stairs_up,
        stairs_down,
        obstacle,
        hurdle,
        opaque,
        opaque_floor,
        last_seen,
        visibility,
    ): (
        Option<&ObjectDefinition>,
        Option<&TextLabel>,
        Option<&Corpse>,
        Option<&Accessible>,
        Option<&StairsUp>,
        Option<&StairsDown>,
        Option<&Obstacle>,
        Option<&Hurdle>,
        Option<&Opaque>,
        Option<&OpaqueFloor>,
        Option<&LastSeen>,
        Option<&Visibility>,
    ),
) -> String {
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

    label.map_or_else(|| String::from("[Unknown]"), String::from)
        + flags
            .iter()
            .map(|s| format!("\n- {s}"))
            .collect::<String>()
            .as_str()
}

fn items_info(
    all_here: &[Entity],
    items: &Query<(Option<&ObjectDefinition>, &TextLabel), (Without<Health>, With<Amount>)>,
) -> String {
    all_here
        .iter()
        .flat_map(|e| items.get(*e))
        .map(|(definition, label)| {
            format!(
                "{label}\n- {:?}\n",
                definition.map_or(ObjectId::new("?"), |d| d.id.clone())
            )
        })
        .collect()
}
