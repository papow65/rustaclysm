use crate::prelude::*;
use bevy::prelude::{Color, Component, EventWriter, NextState, ResMut, States};
use std::fmt;

#[derive(Debug, Component)]
pub(crate) struct Player;

#[derive(Debug)]
enum PlayerBehavior {
    Perform(PlannedAction),
    Feedback(Message),
    NoEffect,
}

/** Current action of the player character

Conceptually, this is a child state of [`GameplayScreenState::Base`]. */
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

    pub(crate) fn plan_action(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut EventWriter<Message>,
        healing_writer: &mut EventWriter<ActorEvent<Healing>>,
        envir: &Envir,
        instruction_queue: &mut InstructionQueue,
        explored: &Explored,
        player: &ActorItem,
        now: Timestamp,
        enemies: &[Pos],
    ) -> Option<PlannedAction> {
        while let Some(instruction) = instruction_queue.pop() {
            match self.plan(next_state, envir, *player.pos, instruction, now) {
                PlayerBehavior::Perform(action) => {
                    return Some(action);
                }
                PlayerBehavior::Feedback(message) => {
                    message_writer.send(message);
                    // Invalid instruction
                    // Continue with next instruction
                }
                PlayerBehavior::NoEffect => {
                    // Valid instruction, but no action performed.
                    // Continue with next instruction
                }
            }
        }

        match self {
            Self::Dragging {
                active_from: Some(from),
            } => auto_drag(envir, instruction_queue, from, enemies),
            Self::AutoDefend => auto_defend(envir, instruction_queue, player, enemies),
            Self::AutoTravel { target } => {
                auto_travel(envir, instruction_queue, explored, player, *target, enemies)
            }
            Self::Pulping {
                active_target: Some(target),
            } => auto_pulp(envir, instruction_queue, player, target, enemies),
            Self::Waiting { until } => auto_wait(instruction_queue, now, until, enemies),
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
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
        now: Timestamp,
    ) -> PlayerBehavior {
        //println!("processing instruction: {instruction:?}");
        match (&self, instruction) {
            (Self::Sleeping { .. }, QueuedInstruction::Finished) => {
                next_state.set(Self::Normal);
                PlayerBehavior::Feedback(Message::info(Phrase::new("You wake up")))
            }
            (Self::Sleeping { .. }, _) => {
                // Can not be interrupted
                PlayerBehavior::Feedback(Message::warn(Phrase::new("You are still asleep. Zzz...")))
            }
            (Self::Normal, QueuedInstruction::Offset(PlayerDirection::Here)) => {
                PlayerBehavior::Perform(PlannedAction::Stay {
                    duration: StayDuration::Short,
                })
            }
            (Self::Normal, QueuedInstruction::Wait) => {
                next_state.set(Self::start_waiting(now));
                PlayerBehavior::Feedback(Message::info(Phrase::new("You wait...")))
            }
            (Self::Normal, QueuedInstruction::Sleep) => {
                next_state.set(Self::start_sleeping(now));
                PlayerBehavior::Feedback(Message::info(Phrase::new("You fall asleep... Zzz...")))
            }
            (Self::Normal, QueuedInstruction::ToggleAutoDefend) => {
                next_state.set(Self::AutoDefend);
                PlayerBehavior::Feedback(Message::info(Phrase::new("You start defending...")))
            }
            (_, QueuedInstruction::ToggleAutoTravel) => PlayerBehavior::Feedback(Message::error(
                Phrase::new("First examine your destination"),
            )),
            (Self::Attacking, QueuedInstruction::Offset(PlayerDirection::Here)) => {
                PlayerBehavior::Feedback(Message::warn(Phrase::new("You can't attack yourself")))
            }
            (Self::Waiting { .. }, QueuedInstruction::Interrupted) => {
                next_state.set(Self::Normal);
                PlayerBehavior::Feedback(Message::warn(Phrase::new(
                    "You spot an enemy and stop waiting",
                )))
            }
            (Self::Waiting { .. }, QueuedInstruction::Finished) => {
                next_state.set(Self::Normal);
                PlayerBehavior::Feedback(Message::info(Phrase::new("Finished waiting")))
            }
            (Self::Pulping { .. }, QueuedInstruction::Interrupted) => {
                next_state.set(Self::Normal);
                PlayerBehavior::Feedback(Message::warn(Phrase::new(
                    "You spot an enemy and stop pulping",
                )))
            }
            (Self::Attacking, QueuedInstruction::Attack)
            | (Self::Smashing, QueuedInstruction::Smash)
            | (Self::Dragging { .. }, QueuedInstruction::Drag | QueuedInstruction::CancelAction)
            | (Self::Pulping { .. }, QueuedInstruction::Finished) => {
                next_state.set(Self::Normal);
                PlayerBehavior::NoEffect
            }
            (
                Self::Dragging {
                    active_from: Some(_),
                },
                QueuedInstruction::Finished,
            ) => {
                next_state.set(Self::Dragging { active_from: None });
                PlayerBehavior::NoEffect
            }
            (
                Self::Dragging {
                    active_from: Some(_),
                },
                _,
            ) => {
                PlayerBehavior::Feedback(Message::warn(Phrase::new("You are still dragging items")))
            }
            (_, instruction) => self.generic_plan(next_state, envir, player_pos, instruction),
        }
    }

    /** For plans that not depend on self */
    #[allow(clippy::needless_pass_by_value)]
    fn generic_plan(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
    ) -> PlayerBehavior {
        //println!("processing generic instruction: {instruction:?}");
        match instruction {
            QueuedInstruction::CancelAction
            | QueuedInstruction::Wait
            | QueuedInstruction::Sleep
            | QueuedInstruction::ToggleAutoTravel
            | QueuedInstruction::ToggleAutoDefend => {
                next_state.set(Self::Normal);
                PlayerBehavior::NoEffect
            }
            QueuedInstruction::Offset(direction) => {
                self.handle_offset(next_state, envir, player_pos, direction.to_nbor())
            }
            QueuedInstruction::Wield(item) => {
                PlayerBehavior::Perform(PlannedAction::Wield { item })
            }
            QueuedInstruction::Unwield(item) => {
                PlayerBehavior::Perform(PlannedAction::Unwield { item })
            }
            QueuedInstruction::Pickup(item) => {
                PlayerBehavior::Perform(PlannedAction::Pickup { item })
            }
            QueuedInstruction::Dump(item, direction) => {
                PlayerBehavior::Perform(PlannedAction::MoveItem {
                    item,
                    to: Nbor::Horizontal(direction),
                })
            }
            QueuedInstruction::Attack => Self::handle_attack(next_state, envir, player_pos),
            QueuedInstruction::Smash => Self::handle_smash(next_state, envir, player_pos),
            QueuedInstruction::Pulp => Self::handle_pulp(next_state, envir, player_pos),
            QueuedInstruction::Close => Self::handle_close(next_state, envir, player_pos),
            QueuedInstruction::Drag => {
                next_state.set(Self::Dragging { active_from: None }); // 'active_from' will temporary be set after moving
                PlayerBehavior::NoEffect
            }
            QueuedInstruction::ExamineItem(item) => {
                PlayerBehavior::Perform(PlannedAction::ExamineItem { item })
            }
            QueuedInstruction::ChangePace => PlayerBehavior::Perform(PlannedAction::ChangePace),
            QueuedInstruction::Interrupted => {
                next_state.set(Self::Normal);
                PlayerBehavior::Feedback(Message::error(Phrase::new(
                    "Iterrupted while not waiting",
                )))
            }
            QueuedInstruction::Finished => {
                next_state.set(Self::Normal);
                PlayerBehavior::Feedback(Message::error(Phrase::new("Finished while not waiting")))
            }
        }
    }

    fn handle_offset(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        envir: &Envir,
        player_pos: Pos,
        raw_nbor: Nbor,
    ) -> PlayerBehavior {
        if !matches!(*self, Self::Sleeping { .. }) {
            if let Err(message) = envir.get_nbor(player_pos, raw_nbor) {
                return PlayerBehavior::Feedback(message);
            }
        }

        match &self {
            Self::Sleeping { .. } => {
                panic!("{self:?} {player_pos:?} {raw_nbor:?}");
            }
            Self::Normal => PlayerBehavior::Perform(PlannedAction::Step { to: raw_nbor }),
            Self::Attacking => {
                next_state.set(Self::Normal);
                PlayerBehavior::Perform(PlannedAction::Attack { target: raw_nbor })
            }
            Self::Smashing => {
                next_state.set(Self::Normal);
                PlayerBehavior::Perform(PlannedAction::Smash { target: raw_nbor })
            }
            Self::Pulping {
                active_target: None,
            } => {
                //eprintln!("Inactive pulping");
                if let Nbor::Horizontal(target) = raw_nbor {
                    //eprintln!("Activating pulping");
                    PlayerBehavior::Perform(PlannedAction::Pulp { target })
                } else {
                    PlayerBehavior::Feedback(Message::warn(Phrase::new(
                        "You can't pulp vertically",
                    )))
                }
            }
            Self::Pulping {
                active_target: Some(target),
            } => {
                panic!("{self:?} {player_pos:?} {raw_nbor:?} {target:?}");
            }
            Self::Closing => {
                next_state.set(Self::Normal);
                if let Nbor::Horizontal(target) = raw_nbor {
                    PlayerBehavior::Perform(PlannedAction::Close { target })
                } else {
                    PlayerBehavior::Feedback(Message::warn(Phrase::new(
                        "You can't close vertically",
                    )))
                }
            }
            Self::Dragging { active_from: None } => {
                next_state.set(Self::Dragging {
                    active_from: Some(player_pos),
                });
                PlayerBehavior::Perform(PlannedAction::Step { to: raw_nbor })
            }
            Self::Dragging {
                active_from: Some(active_from),
            } => {
                panic!("{self:?} {player_pos:?} {raw_nbor:?} {active_from:?}");
            }
            Self::Waiting { .. } | Self::AutoTravel { .. } | Self::AutoDefend => {
                next_state.set(Self::Normal);
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_attack(
        next_state: &mut ResMut<NextState<Self>>,
        envir: &Envir,
        pos: Pos,
    ) -> PlayerBehavior {
        let attackable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Attack)
            .collect::<Vec<_>>();
        match attackable_nbors.len() {
            0 => PlayerBehavior::Feedback(Message::warn(Phrase::new("no targets nearby"))),
            1 => PlayerBehavior::Perform(PlannedAction::Attack {
                target: attackable_nbors[0],
            }),
            _ => {
                next_state.set(Self::Attacking);
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_smash(
        next_state: &mut ResMut<NextState<Self>>,
        envir: &Envir,
        pos: Pos,
    ) -> PlayerBehavior {
        let smashable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Smash)
            .collect::<Vec<_>>();
        match smashable_nbors.len() {
            0 => PlayerBehavior::Feedback(Message::warn(Phrase::new("no targets nearby"))),
            1 => PlayerBehavior::Perform(PlannedAction::Smash {
                target: smashable_nbors[0],
            }),
            _ => {
                next_state.set(Self::Smashing);
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_pulp(
        next_state: &mut ResMut<NextState<Self>>,
        envir: &Envir,
        pos: Pos,
    ) -> PlayerBehavior {
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
            0 => PlayerBehavior::Feedback(Message::warn(Phrase::new("no targets nearby"))),
            1 => {
                //eprintln!("Pulping target found -> active");
                next_state.set(Self::Pulping {
                    active_target: Some(pulpable_nbors[0]),
                });
                PlayerBehavior::Perform(PlannedAction::Pulp {
                    target: pulpable_nbors[0],
                })
            }
            _ => {
                //eprintln!("Pulping choice -> inactive");
                next_state.set(Self::Pulping {
                    active_target: None,
                });
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_close(
        next_state: &mut ResMut<NextState<Self>>,
        envir: &Envir,
        pos: Pos,
    ) -> PlayerBehavior {
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
            0 => PlayerBehavior::Feedback(Message::warn(Phrase::new("nothing to close nearby"))),
            1 => PlayerBehavior::Perform(PlannedAction::Close {
                target: closable_nbors[0],
            }),
            _ => {
                next_state.set(Self::Closing);
                PlayerBehavior::NoEffect
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
            | Self::Dragging { .. }
            | Self::AutoTravel { .. } => WARN_TEXT_COLOR,
            Self::AutoDefend => BAD_TEXT_COLOR,
        }
    }
}

impl fmt::Display for PlayerActionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Normal => "",
            Self::Attacking => "Attacking",
            Self::Smashing => "Smashing",
            Self::Pulping { .. } => "Pulping",
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
    enemies: &[Pos],
) -> Option<PlannedAction> {
    if !enemies.is_empty() {
        instruction_queue.add_interruption();
        None // process the cancellation next turn
    } else if let Some(item) = envir.find_item(*from) {
        Some(PlannedAction::MoveItem {
            item,
            to: Nbor::HERE,
        })
    } else {
        instruction_queue.add_finish();
        None // process the cancellation next turn
    }
}

fn auto_travel(
    envir: &Envir<'_, '_>,
    instruction_queue: &mut InstructionQueue,
    explored: &Explored,
    player: &ActorItem<'_>,
    target: Pos,
    enemies: &[Pos],
) -> Option<PlannedAction> {
    if !enemies.is_empty() || *player.pos == target || player.stamina.breath() != Breath::Normal {
        instruction_queue.add_interruption();
        None // process the cancellation next turn
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
    if enemies.is_empty() || player.stamina.breath() != Breath::Normal {
        instruction_queue.add_interruption();
        None // process the cancellation next turn
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
    target: &HorizontalDirection,
    enemies: &[Pos],
) -> Option<PlannedAction> {
    //eprintln!("Post instruction pulp handling...");
    if !enemies.is_empty() {
        instruction_queue.add_interruption();
        None // process the cancellation next turn
    } else if player.stamina.breath() != Breath::Normal {
        //eprintln!("Keep pulping after catching breath");
        Some(PlannedAction::Stay {
            duration: StayDuration::Short,
        })
    } else if envir
        .find_pulpable(player.pos.horizontal_nbor(*target))
        .is_some()
    {
        //eprintln!("Keep pulping");
        Some(PlannedAction::Pulp { target: *target })
    } else {
        //eprintln!("Stop pulping");
        instruction_queue.add_finish();
        None // process the cancellation next turn
    }
}

fn auto_wait(
    instruction_queue: &mut InstructionQueue,
    now: Timestamp,
    until: &Timestamp,
    enemies: &[Pos],
) -> Option<PlannedAction> {
    if !enemies.is_empty() {
        instruction_queue.add_interruption();
        None // process the cancellation next turn
    } else if *until <= now {
        instruction_queue.add_finish();
        None // process the cancellation next turn
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
        instruction_queue.add_finish();
        None // process the cancellation next turn
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
