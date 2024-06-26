use crate::prelude::*;
use bevy::prelude::{Color, Component, DetectChanges, EventWriter, NextState, ResMut, States};
use std::fmt;

#[derive(Debug, Component)]
pub(crate) struct Player;

/// Current action of the player character
/// Conceptually, this is a child state of [`GameplayScreenState::Base`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, States)]
pub(crate) enum PlayerActionState {
    #[default]
    Normal,
    Attacking,
    Smashing,
    Pulping {
        /// None: intent to pulp on next move
        /// Some: corpse pulping in progress
        active_target: Option<HorizontalDirection>,
    },
    Peeking {
        /// None: intent to peek on next move
        /// Some: validated peeking in progress
        active_target: Option<CardinalDirection>,
    },
    Closing,
    Dragging {
        /// None: intent to drag on next move
        /// Some: moving items from previous to current position
        active_from: Option<Pos>,
    },
    Waiting {
        until: Timestamp,
    },
    Sleeping {
        healing_from: Timestamp,
        until: Timestamp,
    },
    AutoTravel {
        target: Pos,
    },
    AutoDefend,
}

impl PlayerActionState {
    fn start_waiting(now: Timestamp) -> Self {
        Self::Waiting {
            until: now + Milliseconds::MINUTE,
        }
    }

    fn start_sleeping(now: Timestamp) -> Self {
        Self::Sleeping {
            healing_from: now,
            until: now + Milliseconds::EIGHT_HOURS,
        }
    }

    pub(crate) fn plan_manual_action(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        instruction_queue: &mut InstructionQueue,
        player: &ActorItem,
        now: Timestamp,
    ) -> Option<PlannedAction> {
        while let Some(instruction) = instruction_queue.pop() {
            if let Some(action) = self.plan(
                next_state,
                message_writer,
                envir,
                *player.pos,
                instruction,
                now,
            ) {
                return Some(action);
            } else {
                // The action was either an invalid or one that took no time (like [`PlannedAction::ExamineItem`].
                // Continue with next instruction
            }
        }

        None
    }

    pub(crate) fn plan_automatic_action(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        healing_writer: &mut EventWriter<ActorEvent<Healing>>,
        currently_visible_builder: &CurrentlyVisibleBuilder,
        instruction_queue: &mut InstructionQueue,
        explored: &Explored,
        player: &ActorItem,
        now: Timestamp,
        factions: &[(Pos, &Faction)],
    ) -> Option<PlannedAction> {
        let envir = &currently_visible_builder.envir;
        let enemy_name = player
            .faction
            .enemy_name(currently_visible_builder, factions, player);
        match self {
            Self::Dragging {
                active_from: Some(from),
            } => auto_drag(envir, instruction_queue, from, enemy_name),
            Self::AutoDefend => {
                let enemies = player
                    .faction
                    .enemies(currently_visible_builder, factions, player);
                auto_defend(envir, instruction_queue, player, &enemies)
            }
            Self::AutoTravel { target } => auto_travel(
                envir,
                instruction_queue,
                explored,
                player,
                *target,
                enemy_name,
            ),
            Self::Pulping {
                active_target: Some(target),
            } => auto_pulp(envir, instruction_queue, player, *target, enemy_name),
            Self::Waiting { until } => auto_wait(instruction_queue, now, until, enemy_name),
            Self::Sleeping {
                healing_from,
                until,
            } => auto_sleep(
                next_state,
                healing_writer,
                instruction_queue,
                player,
                healing_from,
                now,
                until,
            ),
            _ => {
                instruction_queue.start_waiting();
                println!("Waiting for user action");
                None // no key pressed - wait for the user
            }
        }
    }

    fn plan(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
        now: Timestamp,
    ) -> Option<PlannedAction> {
        //println!("processing instruction: {instruction:?}");
        match (&self, instruction) {
            (Self::Sleeping { .. }, QueuedInstruction::Interrupt(Interruption::Finished)) => {
                next_state.set(Self::Normal);
                message_writer.you("wake up").send_info();
                None
            }
            (Self::Sleeping { .. }, _) => {
                // Can not be interrupted
                message_writer.you("are still asleep. Zzz...").send_error();
                None
            }
            (
                Self::Peeking {
                    active_target: Some(_),
                },
                instruction,
            ) => Self::stop_peeking(
                next_state,
                message_writer,
                envir,
                player_pos,
                instruction,
                now,
            ),
            (Self::Normal, QueuedInstruction::Offset(PlayerDirection::Here)) => {
                Some(PlannedAction::Stay {
                    duration: StayDuration::Short,
                })
            }
            (Self::Normal, QueuedInstruction::Wait) => {
                next_state.set(Self::start_waiting(now));
                message_writer.you("wait...").send_info();
                None
            }
            (Self::Normal, QueuedInstruction::Sleep) => {
                next_state.set(Self::start_sleeping(now));
                message_writer.you("fall asleep... Zzz...").send_info();
                None
            }
            (Self::Normal, QueuedInstruction::ToggleAutoDefend) => {
                next_state.set(Self::AutoDefend);
                message_writer.you("start defending...").send_warn();
                None
            }
            (_, QueuedInstruction::ToggleAutoTravel) => {
                message_writer
                    .str("First examine your destination")
                    .send_error();
                None
            }
            (Self::Attacking, QueuedInstruction::Offset(PlayerDirection::Here)) => {
                message_writer.you("can't attack yourself").send_error();
                None
            }
            (Self::Attacking, QueuedInstruction::Attack)
            | (Self::Smashing, QueuedInstruction::Smash)
            | (Self::Peeking { .. }, QueuedInstruction::Peek)
            | (Self::Dragging { .. }, QueuedInstruction::Drag | QueuedInstruction::CancelAction)
            | (Self::Pulping { .. }, QueuedInstruction::Interrupt(Interruption::Finished)) => {
                next_state.set(Self::Normal);
                None
            }
            (
                Self::Dragging {
                    active_from: Some(_),
                },
                QueuedInstruction::Interrupt(Interruption::Finished),
            ) => {
                next_state.set(Self::Dragging { active_from: None });
                None
            }
            (
                Self::Dragging {
                    active_from: Some(_),
                },
                _,
            ) => {
                message_writer.you("are still dragging items").send_warn();
                None
            }
            (_, instruction) => {
                self.generic_plan(next_state, message_writer, envir, player_pos, instruction)
            }
        }
    }

    /// For plans that not depend on self
    #[allow(clippy::needless_pass_by_value)]
    fn generic_plan(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
    ) -> Option<PlannedAction> {
        //println!("processing generic instruction: {instruction:?}");
        match instruction {
            QueuedInstruction::CancelAction
            | QueuedInstruction::Wait
            | QueuedInstruction::Sleep
            | QueuedInstruction::ToggleAutoTravel
            | QueuedInstruction::ToggleAutoDefend => {
                next_state.set(Self::Normal);
                None
            }
            QueuedInstruction::Offset(direction) => self.handle_offset(
                next_state,
                message_writer,
                envir,
                player_pos,
                direction.to_nbor(),
            ),
            QueuedInstruction::Wield(item) => Some(PlannedAction::Wield { item }),
            QueuedInstruction::Unwield(item) => Some(PlannedAction::Unwield { item }),
            QueuedInstruction::Pickup(item) => Some(PlannedAction::Pickup { item }),
            QueuedInstruction::Dump(item, direction) => Some(PlannedAction::MoveItem {
                item,
                to: Nbor::Horizontal(direction),
            }),
            QueuedInstruction::Attack => {
                Self::handle_attack(next_state, message_writer, envir, player_pos)
            }
            QueuedInstruction::Smash => {
                Self::handle_smash(next_state, message_writer, envir, player_pos)
            }
            QueuedInstruction::Pulp => {
                Self::handle_pulp(next_state, message_writer, envir, player_pos)
            }
            QueuedInstruction::Peek => {
                next_state.set(Self::Peeking {
                    active_target: None,
                });
                None
            }
            QueuedInstruction::Close => {
                Self::handle_close(next_state, message_writer, envir, player_pos)
            }
            QueuedInstruction::Drag => {
                next_state.set(Self::Dragging { active_from: None }); // 'active_from' will temporary be set after moving
                None
            }
            QueuedInstruction::ExamineItem(item) => Some(PlannedAction::ExamineItem { item }),
            QueuedInstruction::ChangePace => Some(PlannedAction::ChangePace),
            QueuedInstruction::Interrupt(Interruption::Danger(fragments)) => {
                next_state.set(Self::Normal);
                message_writer
                    .you("spot")
                    .push(fragments)
                    .add("and stop")
                    .add(self.to_string().to_lowercase())
                    .send_warn();
                None
            }
            QueuedInstruction::Interrupt(Interruption::LowStamina) => {
                next_state.set(Self::Normal);
                message_writer
                    .you("are almost out of breath and stop")
                    .add(self.to_string().to_lowercase())
                    .send_warn();
                None
            }
            QueuedInstruction::Interrupt(Interruption::Finished) => {
                next_state.set(Self::Normal);
                message_writer
                    .you("finish")
                    .add(self.to_string().to_lowercase())
                    .send_info();
                None
            }
        }
    }

    fn handle_offset(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        player_pos: Pos,
        raw_nbor: Nbor,
    ) -> Option<PlannedAction> {
        if !matches!(*self, Self::Sleeping { .. }) {
            if let Err(failure) = envir.get_nbor(player_pos, raw_nbor) {
                message_writer.str(failure).send_error();
                return None;
            }
        }

        match &self {
            Self::Sleeping { .. }
            | Self::Peeking {
                active_target: Some(_),
            }
            | Self::Dragging {
                active_from: Some(_),
            } => {
                panic!("{self:?} {player_pos:?} {raw_nbor:?}");
            }
            Self::Normal => Some(PlannedAction::Step { to: raw_nbor }),
            Self::Attacking => {
                next_state.set(Self::Normal);
                Some(PlannedAction::Attack { target: raw_nbor })
            }
            Self::Smashing => {
                next_state.set(Self::Normal);
                Some(PlannedAction::Smash { target: raw_nbor })
            }
            Self::Pulping {
                active_target: None,
            } => {
                //eprintln!("Inactive pulping");
                if let Nbor::Horizontal(target) = raw_nbor {
                    //eprintln!("Activating pulping");
                    Some(PlannedAction::Pulp { target })
                } else {
                    message_writer.you("can't pulp vertically").send_error();
                    None
                }
            }
            Self::Pulping {
                active_target: Some(target),
            } => {
                panic!("{self:?} {player_pos:?} {raw_nbor:?} {target:?}");
            }
            Self::Peeking {
                active_target: None,
            } => {
                match raw_nbor {
                    Nbor::Up | Nbor::Down => {
                        message_writer.you("can't peek vertically").send_error();
                        None
                    }
                    Nbor::HERE => {
                        message_writer.you("can't peek here").send_error();
                        None
                    }
                    Nbor::Horizontal(
                        HorizontalDirection::NorthWest
                        | HorizontalDirection::SouthWest
                        | HorizontalDirection::NorthEast
                        | HorizontalDirection::SouthEast,
                    ) => {
                        message_writer.you("can't peek diagonally").send_error();
                        None
                    }
                    Nbor::Horizontal(
                        horizontal_direction @ (HorizontalDirection::North
                        | HorizontalDirection::South
                        | HorizontalDirection::East
                        | HorizontalDirection::West),
                    ) => {
                        let target = CardinalDirection::try_from(horizontal_direction)
                            .unwrap_or_else(|()| {
                                panic!(
                                    "{horizontal_direction:?} should match a cardinal direction"
                                );
                            });
                        //eprintln!("Activating peeking");
                        // Not Self::Peeking, because that is not validated yet.
                        next_state.set(Self::Normal);
                        Some(PlannedAction::Peek { target })
                    }
                }
            }
            Self::Closing => {
                next_state.set(Self::Normal);
                if let Nbor::Horizontal(target) = raw_nbor {
                    Some(PlannedAction::Close { target })
                } else {
                    message_writer.you("can't close vertically").send_error();
                    None
                }
            }
            Self::Dragging { active_from: None } => {
                next_state.set(Self::Dragging {
                    active_from: Some(player_pos),
                });
                Some(PlannedAction::Step { to: raw_nbor })
            }
            Self::Waiting { .. } | Self::AutoTravel { .. } | Self::AutoDefend => {
                next_state.set(Self::Normal);
                None
            }
        }
    }

    fn handle_attack(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        pos: Pos,
    ) -> Option<PlannedAction> {
        let attackable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Attack)
            .collect::<Vec<_>>();
        match attackable_nbors.len() {
            0 => {
                message_writer.str("no targets nearby").send_error();
                None
            }
            1 => Some(PlannedAction::Attack {
                target: attackable_nbors[0],
            }),
            _ => {
                next_state.set(Self::Attacking);
                None
            }
        }
    }

    fn handle_smash(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        pos: Pos,
    ) -> Option<PlannedAction> {
        let smashable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Smash)
            .collect::<Vec<_>>();
        match smashable_nbors.len() {
            0 => {
                message_writer.str("no targets nearby").send_error();
                None
            }
            1 => Some(PlannedAction::Smash {
                target: smashable_nbors[0],
            }),
            _ => {
                next_state.set(Self::Smashing);
                None
            }
        }
    }

    fn handle_pulp(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        pos: Pos,
    ) -> Option<PlannedAction> {
        let pulpable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Pulp)
            .filter_map(|nbor| {
                if let Nbor::Horizontal(horizontal) = nbor {
                    Some(horizontal)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        //eprintln!("Pulping {} targets", pulpable_nbors.len());
        match pulpable_nbors.len() {
            0 => {
                message_writer.str("no targets nearby").send_error();
                None
            }
            1 => {
                //eprintln!("Pulping target found -> active");
                next_state.set(Self::Pulping {
                    active_target: Some(pulpable_nbors[0]),
                });
                Some(PlannedAction::Pulp {
                    target: pulpable_nbors[0],
                })
            }
            _ => {
                //eprintln!("Pulping choice -> inactive");
                next_state.set(Self::Pulping {
                    active_target: None,
                });
                None
            }
        }
    }

    fn handle_close(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        pos: Pos,
    ) -> Option<PlannedAction> {
        let closable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Close)
            .filter_map(|nbor| {
                if let Nbor::Horizontal(horizontal) = nbor {
                    Some(horizontal)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        match closable_nbors.len() {
            0 => {
                message_writer.str("nothing to close nearby").send_error();
                None
            }
            1 => Some(PlannedAction::Close {
                target: closable_nbors[0],
            }),
            _ => {
                next_state.set(Self::Closing);
                None
            }
        }
    }

    pub(crate) const fn color(&self) -> Color {
        match self {
            Self::Normal | Self::Closing => DEFAULT_TEXT_COLOR,
            Self::Waiting { .. }
            | Self::Sleeping { .. }
            | Self::Attacking
            | Self::Smashing
            | Self::Pulping { .. }
            | Self::Peeking { .. }
            | Self::Dragging { .. }
            | Self::AutoTravel { .. } => WARN_TEXT_COLOR,
            Self::AutoDefend => BAD_TEXT_COLOR,
        }
    }

    fn stop_peeking(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
        now: Timestamp,
    ) -> Option<PlannedAction> {
        // Pretend to be Self::Normal
        Self::Normal
            .plan(
                next_state,
                message_writer,
                envir,
                player_pos,
                instruction,
                now,
            )
            .inspect(
                // Only called for Some(planned_action)
                |_| {
                    if !next_state.is_changed() {
                        next_state.set(Self::Normal);
                    }
                },
            )
    }
}

impl fmt::Display for PlayerActionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Normal => "",
            Self::Attacking => "Attacking",
            Self::Smashing => "Smashing",
            Self::Pulping { .. } => "Pulping",
            Self::Peeking { .. } => "Peeking",
            Self::Closing => "Closing",
            Self::Dragging { .. } => "Dragging",
            Self::Waiting { .. } => "Waiting",
            Self::Sleeping { .. } => "Sleeping",
            Self::AutoTravel { .. } => "Traveling",
            Self::AutoDefend => "Defending",
        })
    }
}

fn auto_drag(
    envir: &Envir<'_, '_>,
    instruction_queue: &mut InstructionQueue,
    from: &Pos,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    if let Some(item) = envir.find_item(*from) {
        if let Some(enemy_name) = enemy_name {
            instruction_queue.interrupt(Interruption::Danger(enemy_name));
            None
        } else {
            Some(PlannedAction::MoveItem {
                item,
                to: Nbor::HERE,
            })
        }
    } else {
        instruction_queue.interrupt(Interruption::Finished);
        None
    }
}

fn auto_travel(
    envir: &Envir<'_, '_>,
    instruction_queue: &mut InstructionQueue,
    explored: &Explored,
    player: &ActorItem<'_>,
    target: Pos,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    if *player.pos == target {
        instruction_queue.interrupt(Interruption::Finished);
        None
    } else if let Some(enemy_name) = enemy_name {
        instruction_queue.interrupt(Interruption::Danger(enemy_name));
        None
    } else if player.stamina.breath() != Breath::Normal {
        instruction_queue.interrupt(Interruption::LowStamina);
        None
    } else {
        envir
            .path(
                *player.pos,
                target,
                Intelligence::Smart,
                |pos| explored.has_pos_been_seen(pos),
                player.speed(),
            )
            .map(|path| {
                envir
                    .nbor(*player.pos, path.first)
                    .expect("The first step should be a nbor")
            })
            .or_else(|| {
                // Full path not available
                envir
                    .nbors_for_moving(*player.pos, None, Intelligence::Smart, player.speed())
                    .map(|(nbor, nbor_pos, _)| (nbor, nbor_pos.vision_distance(target)))
                    .min_by_key(|(_, distance)| *distance)
                    .filter(|(_, distance)| *distance < player.pos.vision_distance(target))
                    .map(|(nbor, _)| nbor)
            })
            .map(|to| PlannedAction::Step { to })
    }
}

fn auto_defend(
    envir: &Envir<'_, '_>,
    instruction_queue: &mut InstructionQueue,
    player: &ActorItem<'_>,
    enemies: &[Pos],
) -> Option<PlannedAction> {
    if enemies.is_empty() {
        instruction_queue.interrupt(Interruption::Finished);
        None
    } else if player.stamina.breath() != Breath::Normal {
        instruction_queue.interrupt(Interruption::LowStamina);
        None
    } else if let Some(target) = enemies.iter().find_map(|pos| envir.nbor(*player.pos, *pos)) {
        Some(PlannedAction::Attack { target })
    } else {
        Some(PlannedAction::Stay {
            duration: StayDuration::Short,
        })
    }
}

fn auto_pulp(
    envir: &Envir<'_, '_>,
    instruction_queue: &mut InstructionQueue,
    player: &ActorItem<'_>,
    target: HorizontalDirection,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    //eprintln!("Post instruction pulp handling...");
    if envir
        .find_pulpable(player.pos.horizontal_nbor(target))
        .is_some()
    {
        if let Some(enemy_name) = enemy_name {
            instruction_queue.interrupt(Interruption::Danger(enemy_name));
            None
        } else if player.stamina.breath() != Breath::Normal {
            //eprintln!("Keep pulping after catching breath");
            Some(PlannedAction::Stay {
                duration: StayDuration::Short,
            })
        } else {
            //eprintln!("Keep pulping");
            Some(PlannedAction::Pulp { target })
        }
    } else {
        //eprintln!("Stop pulping");
        instruction_queue.interrupt(Interruption::Finished);
        None
    }
}

fn auto_wait(
    instruction_queue: &mut InstructionQueue,
    now: Timestamp,
    until: &Timestamp,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    if *until <= now {
        instruction_queue.interrupt(Interruption::Finished);
        None
    } else if let Some(enemy_name) = enemy_name {
        instruction_queue.interrupt(Interruption::Danger(enemy_name));
        None
    } else {
        Some(PlannedAction::Stay {
            duration: StayDuration::Short,
        })
    }
}

fn auto_sleep(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    healing_writer: &mut EventWriter<'_, ActorEvent<Healing>>,
    instruction_queue: &mut InstructionQueue,
    player: &ActorItem<'_>,
    healing_from: &Timestamp,
    now: Timestamp,
    until: &Timestamp,
) -> Option<PlannedAction> {
    // TODO interrupt on taking damage

    assert!(healing_from < until, "{healing_from:?} < {until:?}");
    assert!(*healing_from <= now, "{healing_from:?} <= {now:?}");
    //eprintln!("{healing_from:?} <= {now:?}");
    let sleeping_duration = now - *healing_from;

    let healing_amount = sleeping_duration.0 / 1_000_000;
    healing_writer.send(ActorEvent::new(
        player.entity,
        Healing {
            amount: healing_amount as u16,
        },
    ));

    if *until <= now {
        instruction_queue.interrupt(Interruption::Finished);
        None
    } else {
        let healing_duration = Milliseconds(healing_amount * 1_000_000);
        // dbg!(healing_from);
        next_state.set(PlayerActionState::Sleeping {
            healing_from: *healing_from + healing_duration,
            until: *until,
        });
        Some(PlannedAction::Stay {
            duration: StayDuration::Long,
        })
    }
}
