use super::log_if_slow;
use crate::prelude::*;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use chrono::prelude::{Datelike, Local};
use std::collections::BTreeMap;
use std::time::Instant;

pub(crate) struct HudDefaults {
    text_style: TextStyle,
}

impl HudDefaults {
    pub(crate) fn new(asset_server: &mut AssetServer) -> Self {
        Self {
            text_style: TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.otf"),
                font_size: 16.0,
                color: Color::rgb(0.8, 0.8, 0.8),
            },
        }
    }
}

fn spawn_log_display(parent: &mut EntityCommands) {
    // TODO properly use flex layout
    parent.with_children(|child_builder| {
        child_builder
            .spawn_bundle(TextBundle {
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
                    margin: UiRect::all(Val::Px(5.0)),
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
            .spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "\n".to_string(),
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
        .spawn_bundle(background)
        .insert(ManualDisplay)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "move         numpad\nup/down      r/f\npick/drop    b/v\nattack       a\nrun          +\nexamine      x\nexamine map  m\nzoom         (shift+)z\nzoom         scroll wheel\ntoggle this  h\nquit         ctrl+c/d/q".to_string(),
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
pub(crate) fn spawn_hud(mut commands: Commands, mut asset_server: ResMut<AssetServer>) {
    let hud_defaults = HudDefaults::new(asset_server.as_mut());

    let mut background = NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        color: Color::rgba(0.25, 0.25, 0.25, 0.6).into(),
        ..default()
    };

    spawn_manual_display(&mut commands, &hud_defaults, background.clone());

    background.style.position.top = Val::Px(0.0);
    background.style.position.right = Val::Px(0.0);
    background.style.size = Size {
        width: Val::Px(353.0), // for 43 chars - determined by trial and error
        height: Val::Percent(100.0),
    };
    let mut parent = commands.spawn_bundle(background);
    spawn_status_display(&hud_defaults, &mut parent);
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
        println!("{string}", string = message.0);
        new_messages = true;
    }
    if !new_messages {
        return;
    }

    let sections = &mut logs.iter_mut().next().unwrap().sections;
    sections.clear();
    for message in messages
        .iter()
        .collect::<Vec<&Message>>()
        .iter()
        .rev()
        .take(20)
        .rev()
    {
        let mut style = hud_defaults.text_style.clone();
        style.color = message.1;
        sections.push(TextSection {
            value: format!("{string}\n", string = message.0),
            style,
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

    let season_length = 91; // TODO look up

    let tenth_seconds = timeouts.time().0 / 100;
    let seconds = tenth_seconds / 10;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let seasons = days / season_length;
    let years = seasons / 4;

    status_displays.iter_mut().next().unwrap().sections[1].value = format!(
        "{:#04}-{}-{:#02} {:#02}:{:#02}:{:#02}.{}\n",
        years + Local::now().year() as u64 + 1, // based on https://cataclysmdda.org/lore-background.html
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
        status_displays.iter_mut().next().unwrap().sections[2].value =
            format!("{} health\n", health);
    }

    log_if_slow("update_status_health", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_speed(
    speed: Query<&Speed, (With<Player>, Changed<Speed>)>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Some(speed) = speed.iter().next() {
        status_displays.iter_mut().next().unwrap().sections[3].value = format!("{}\n", speed.h);
    }

    log_if_slow("update_status_speed", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_player_state(
    player: Query<&Player, Changed<Player>>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Some(player) = player.iter().next() {
        status_displays.iter_mut().next().unwrap().sections[4].value =
            format!("{}\n", player.state);
    }

    log_if_slow("update_status_player_state", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_status_detais(
    envir: Envir,
    mut labels: ResMut<ZoneLevelNames>,
    characters: Query<(Option<&Label>, &Health), Without<Item>>,
    entities: Query<
        (
            Option<&Label>,
            Option<&Health>,
            Option<&Corpse>,
            Option<&Action>,
            Option<&Floor>,
            Option<&Stairs>,
            Option<&Obstacle>,
            Option<&Hurdle>,
            Option<&Opaque>,
        ),
        (Without<Health>, Without<Item>),
    >,
    items: Query<(Option<&Label>, &Item), Without<Health>>,
    player: Query<&Player, Changed<Player>>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
    globs: Query<(&GlobalTransform, Option<&ZoneLevel>)>,
) {
    let start = Instant::now();

    if let Some(player) = player.iter().next() {
        status_displays.iter_mut().next().unwrap().sections[5].value = match player.state {
            PlayerActionState::ExaminingPos(pos) => {
                for ent in envir.location.all(pos) {
                    if let Ok((glob, _)) = globs.get(ent) {
                        println!("Global transfrom: {glob:?}");
                    }
                }

                pos_info(&envir, &characters, &entities, &items, pos)
            }
            PlayerActionState::ExaminingZoneLevel(zone_level) => {
                for (glob, z) in globs.iter() {
                    if z == Some(&zone_level) {
                        println!("Global transfrom: {glob:?}");
                    }
                }

                format!(
                    "{:?}\n{:?}",
                    zone_level,
                    labels
                        .get(zone_level)
                        .unwrap_or(&ObjectName::new("NOT FOUND"))
                )
            }
            _ => String::new(),
        };
    }

    log_if_slow("update_status_detais", start);
}

fn pos_info(
    envir: &Envir,
    characters: &Query<(Option<&Label>, &Health), Without<Item>>,
    entities: &Query<
        (
            Option<&Label>,
            Option<&Health>,
            Option<&Corpse>,
            Option<&Action>,
            Option<&Floor>,
            Option<&Stairs>,
            Option<&Obstacle>,
            Option<&Hurdle>,
            Option<&Opaque>,
        ),
        (Without<Health>, Without<Item>),
    >,
    items: &Query<(Option<&Label>, &Item), Without<Health>>,
    pos: Pos,
) -> String {
    let all_here = envir.location.all(pos);

    let characters = all_here
        .iter()
        .flat_map(|&i| characters.get(i))
        .map(|(label, health)| {
            let label = label.map_or_else(|| "?".to_string(), |l| l.0.clone());
            format!("{} ({} health)\n", label, health)
        })
        .collect::<String>();

    let entities = all_here
        .iter()
        .flat_map(|&i| entities.get(i))
        .map(
            |(label, health, corpse, action, floor, stairs, obstacle, hurdle, opaque)| {
                let label = label.map_or_else(|| "?".to_string(), |l| l.0.clone());
                let mut flags = Vec::new();
                let health_str;
                if let Some(health) = health {
                    health_str = format!("health({})", health);
                    flags.push(health_str.as_str());
                }
                if corpse.is_some() {
                    flags.push("corpse");
                }
                let action_str;
                if let Some(action) = action {
                    action_str = format!("{:?}", action);
                    flags.push(action_str.as_str());
                }
                if floor.is_some() {
                    flags.push("floor");
                }
                if stairs.is_some() {
                    flags.push("stairs");
                }
                if obstacle.is_some() {
                    flags.push("obstacle");
                }
                let hurdle_str;
                if let Some(hurdle) = hurdle {
                    hurdle_str = format!("hurdle({})", hurdle.0);
                    flags.push(hurdle_str.as_str());
                }
                if opaque.is_some() {
                    flags.push("opaque");
                }
                label
                    + flags
                        .iter()
                        .map(|s| format!("\n- {}", s))
                        .collect::<String>()
                        .as_str()
                    + "\n"
            },
        )
        .collect::<String>();

    let mut grouped_items = BTreeMap::<Option<&Label>, u32>::new();
    for (label, item) in all_here.iter().flat_map(|&i| items.get(i)) {
        *grouped_items.entry(label).or_insert(0) += item.amount;
    }
    let items = grouped_items
        .iter()
        .map(|(label, amount)| {
            let label = label.map_or_else(|| "?".to_string(), |l| l.0.clone());
            let amount = Some(amount)
                .filter(|&&a| 1 < a)
                .map(|a| format!(" (x{})", a))
                .unwrap_or_default();
            label + amount.as_str() + "\n"
        })
        .collect::<String>();

    format!(
        "{:?}\n{}{}{}{}",
        pos,
        characters,
        if envir.has_stairs_down(pos) {
            "Stairs down\n"
        } else {
            ""
        },
        entities,
        items
    )
}
