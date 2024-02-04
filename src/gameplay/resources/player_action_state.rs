use crate::prelude::*;
use bevy::prelude::{Color, EventWriter, Resource};
use std::fmt;

#[derive(Debug)]
enum PlayerBehavior {
    Perform(PlannedAction),
    Feedback(Message),
    NoEffect,
}

/** Conceptually, this is a child state of `GameplayScreenState::Base`

Not a bevy state because that doesn't suport enum values with fields */
#[derive(Debug, Default, PartialEq, Eq, Resource)]
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
    ExaminingPos(Pos),
    ExaminingZoneLevel(ZoneLevel),
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

    pub(crate) const fn cancel_context(&self) -> CancelContext {
        match self {
            Self::Normal | Self::Sleeping { .. } => CancelContext::State,
            _ => CancelContext::Action,
        }
    }

    pub(crate) fn plan_action(
        &mut self,
        message_writer: &mut EventWriter<Message>,
        healing_writer: &mut EventWriter<ActionEvent<Healing>>,
        envir: &mut Envir,
        instruction_queue: &mut InstructionQueue,
        player: &ActorItem,
        now: Timestamp,
        enemies: &[Pos],
    ) -> Option<PlannedAction> {
        while let Some(instruction) = instruction_queue.pop() {
            match self.plan(envir, *player.pos, instruction, now) {
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
            Self::Pulping {
                active_target: Some(target),
            } => {
                //eprintln!("Post instruction pulp handling...");
                if !enemies.is_empty() {
                    instruction_queue.add_interruption();
                    None // process the cancellation next turn
                } else if player.stamina.breath() == Breath::Winded {
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
            Self::Dragging {
                active_from: Some(from),
            } => {
                if let Some(item) = envir.find_item(*from) {
                    Some(PlannedAction::MoveItem {
                        item,
                        to: Nbor::HERE,
                    })
                } else {
                    instruction_queue.add_finish();
                    None // process the cancellation next turn
                }
            }
            Self::Waiting { until } => {
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
            Self::Sleeping {
                healing_from,
                until,
            } => {
                // TODO interrupt on taking damage

                assert!(healing_from < until, "{healing_from:?} < {until:?}");
                assert!(*healing_from <= now, "{healing_from:?} <= {now:?}");
                //eprintln!("{healing_from:?} <= {now:?}");
                let sleeping_duration = now - *healing_from;

                let healing_amount = sleeping_duration.0 / 1_000_000;
                healing_writer.send(ActionEvent::new(
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
                    *healing_from += healing_duration;
                    // dbg!(healing_from);

                    Some(PlannedAction::Stay {
                        duration: StayDuration::Long,
                    })
                }
            }
            _ => {
                instruction_queue.start_waiting();
                println!("Waiting for user action");
                None // no key pressed - wait for the user
            }
        }
    }

    fn plan(
        &mut self,
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
        now: Timestamp,
    ) -> PlayerBehavior {
        //println!("processing instruction: {instruction:?}");
        assert!(
            instruction != QueuedInstruction::CancelAction
                || self.cancel_context() == CancelContext::Action,
            "{self:?} is not an action to cancel"
        );
        match (&self, instruction) {
            (Self::Sleeping { .. }, QueuedInstruction::Finished) => {
                *self = Self::Normal;
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
                *self = Self::start_waiting(now);
                PlayerBehavior::Feedback(Message::info(Phrase::new("You wait...")))
            }
            (Self::Normal, QueuedInstruction::Sleep) => {
                *self = Self::start_sleeping(now);
                PlayerBehavior::Feedback(Message::info(Phrase::new("You fall asleep... Zzz...")))
            }
            (Self::Attacking, QueuedInstruction::Offset(PlayerDirection::Here)) => {
                PlayerBehavior::Feedback(Message::warn(Phrase::new("You can't attack yourself")))
            }
            (Self::Waiting { .. }, QueuedInstruction::Interrupted) => {
                *self = Self::Normal;
                PlayerBehavior::Feedback(Message::warn(Phrase::new(
                    "You spot an enemy and stop waiting",
                )))
            }
            (Self::Waiting { .. }, QueuedInstruction::Finished) => {
                *self = Self::Normal;
                PlayerBehavior::Feedback(Message::info(Phrase::new("Finished waiting")))
            }
            (Self::Pulping { .. }, QueuedInstruction::Interrupted) => {
                *self = Self::Normal;
                PlayerBehavior::Feedback(Message::warn(Phrase::new(
                    "You spot an enemy and stop pulping",
                )))
            }
            (Self::Attacking, QueuedInstruction::Attack)
            | (Self::Smashing, QueuedInstruction::Smash)
            | (Self::Dragging { .. }, QueuedInstruction::Drag | QueuedInstruction::CancelAction)
            | (Self::Pulping { .. }, QueuedInstruction::Finished)
            | (Self::ExaminingPos(_), QueuedInstruction::ExaminePos)
            | (Self::ExaminingZoneLevel(_), QueuedInstruction::ExamineZoneLevel) => {
                *self = Self::Normal;
                PlayerBehavior::NoEffect
            }
            (
                Self::Dragging {
                    active_from: Some(_),
                },
                QueuedInstruction::Finished,
            ) => {
                *self = Self::Dragging { active_from: None };
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
            (_, instruction) => self.generic_plan(envir, player_pos, instruction),
        }
    }

    /** For plans that not depend on self */
    #[allow(clippy::needless_pass_by_value)]
    fn generic_plan(
        &mut self,
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
    ) -> PlayerBehavior {
        //println!("processing generic instruction: {instruction:?}");
        match instruction {
            QueuedInstruction::CancelAction
            | QueuedInstruction::Wait
            | QueuedInstruction::Sleep => {
                *self = Self::Normal;
                PlayerBehavior::NoEffect
            }
            QueuedInstruction::Offset(direction) => {
                self.handle_offset(envir, player_pos, direction.to_nbor())
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
            QueuedInstruction::Attack => self.handle_attack(envir, player_pos),
            QueuedInstruction::Smash => self.handle_smash(envir, player_pos),
            QueuedInstruction::Pulp => self.handle_pulp(envir, player_pos),
            QueuedInstruction::Close => self.handle_close(envir, player_pos),
            QueuedInstruction::Drag => {
                *self = Self::Dragging { active_from: None }; // 'active_from' will temporary be set after moving
                PlayerBehavior::NoEffect
            }
            QueuedInstruction::ExamineItem(item) => {
                PlayerBehavior::Perform(PlannedAction::ExamineItem { item })
            }
            QueuedInstruction::ExaminePos => {
                let target = Pos::from(&Focus::new(self, player_pos));
                *self = Self::ExaminingPos(target);
                PlayerBehavior::NoEffect
            }
            QueuedInstruction::ExamineZoneLevel => {
                let target = ZoneLevel::from(&Focus::new(self, player_pos));
                *self = Self::ExaminingZoneLevel(target);
                PlayerBehavior::NoEffect
            }
            QueuedInstruction::ChangePace => PlayerBehavior::Perform(PlannedAction::ChangePace),
            QueuedInstruction::Interrupted => {
                *self = Self::Normal;
                PlayerBehavior::Feedback(Message::error(Phrase::new(
                    "Iterrupted while not waiting",
                )))
            }
            QueuedInstruction::Finished => {
                *self = Self::Normal;
                PlayerBehavior::Feedback(Message::error(Phrase::new("Finished while not waiting")))
            }
        }
    }

    fn handle_offset(&mut self, envir: &Envir, player_pos: Pos, raw_nbor: Nbor) -> PlayerBehavior {
        if !matches!(
            *self,
            Self::Sleeping { .. } | Self::ExaminingZoneLevel(_) | Self::ExaminingPos(_)
        ) {
            if let Err(message) = envir.get_nbor(player_pos, raw_nbor) {
                return PlayerBehavior::Feedback(message);
            }
        }

        match &self {
            Self::Sleeping { .. } => {
                panic!("{self:?} {player_pos:?} {raw_nbor:?}");
            }
            Self::ExaminingZoneLevel(current) => {
                if let Some(target) = current.nbor(raw_nbor) {
                    *self = Self::ExaminingZoneLevel(target);
                    PlayerBehavior::NoEffect
                } else {
                    PlayerBehavior::Feedback(Message::warn(Phrase::new(
                        "invalid zone level to examine",
                    )))
                }
            }
            Self::ExaminingPos(current) => {
                if let Some(target) = current.raw_nbor(raw_nbor) {
                    *self = Self::ExaminingPos(target);
                    PlayerBehavior::NoEffect
                } else {
                    PlayerBehavior::Feedback(Message::warn(Phrase::new(
                        "invalid position to examine",
                    )))
                }
            }
            Self::Normal => PlayerBehavior::Perform(PlannedAction::Step { to: raw_nbor }),
            Self::Attacking => {
                *self = Self::Normal;
                PlayerBehavior::Perform(PlannedAction::Attack { target: raw_nbor })
            }
            Self::Smashing => {
                *self = Self::Normal;
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
                *self = Self::Normal;
                if let Nbor::Horizontal(target) = raw_nbor {
                    PlayerBehavior::Perform(PlannedAction::Close { target })
                } else {
                    PlayerBehavior::Feedback(Message::warn(Phrase::new(
                        "You can't close vertically",
                    )))
                }
            }
            Self::Dragging { active_from: None } => {
                *self = Self::Dragging {
                    active_from: Some(player_pos),
                };
                PlayerBehavior::Perform(PlannedAction::Step { to: raw_nbor })
            }
            Self::Dragging {
                active_from: Some(active_from),
            } => {
                panic!("{self:?} {player_pos:?} {raw_nbor:?} {active_from:?}");
            }
            Self::Waiting { .. } => {
                *self = Self::Normal;
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_attack(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let attackable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Attack)
            .collect::<Vec<_>>();
        match attackable_nbors.len() {
            0 => PlayerBehavior::Feedback(Message::warn(Phrase::new("no targets nearby"))),
            1 => PlayerBehavior::Perform(PlannedAction::Attack {
                target: attackable_nbors[0],
            }),
            _ => {
                *self = Self::Attacking;
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_smash(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let smashable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Smash)
            .collect::<Vec<_>>();
        match smashable_nbors.len() {
            0 => PlayerBehavior::Feedback(Message::warn(Phrase::new("no targets nearby"))),
            1 => PlayerBehavior::Perform(PlannedAction::Smash {
                target: smashable_nbors[0],
            }),
            _ => {
                *self = Self::Smashing;
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_pulp(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
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
                *self = Self::Pulping {
                    active_target: Some(pulpable_nbors[0]),
                };
                PlayerBehavior::Perform(PlannedAction::Pulp {
                    target: pulpable_nbors[0],
                })
            }
            _ => {
                //eprintln!("Pulping choice -> inactive");
                *self = Self::Pulping {
                    active_target: None,
                };
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_close(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
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
                *self = Self::Closing;
                PlayerBehavior::NoEffect
            }
        }
    }

    pub(crate) const fn color(&self) -> Color {
        match self {
            Self::Normal | Self::Closing | Self::ExaminingPos(_) | Self::ExaminingZoneLevel(_) => {
                DEFAULT_TEXT_COLOR
            }
            Self::Waiting { .. }
            | Self::Sleeping { .. }
            | Self::Attacking
            | Self::Smashing
            | Self::Pulping { .. }
            | Self::Dragging { .. } => WARN_TEXT_COLOR,
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
            Self::ExaminingPos(_) => "Examining",
            Self::ExaminingZoneLevel(_) => "Examining map",
        })
    }
}
