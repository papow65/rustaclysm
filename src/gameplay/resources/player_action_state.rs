use crate::prelude::*;
use bevy::prelude::{Color, Commands, Entity, EventWriter, Resource};
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
    Closing,
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
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        envir: &mut Envir,
        instruction_queue: &mut InstructionQueue,
        actor: Entity,
        pos: Pos,
        now: Timestamp,
        enemies: &[Pos],
    ) -> Option<PlannedAction> {
        while let Some(instruction) = instruction_queue.pop() {
            match self.plan(envir, pos, instruction, now) {
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
            Self::Waiting { until } => {
                if !enemies.is_empty() {
                    instruction_queue.add(QueuedInstruction::Interrupted);
                    None // process the cancellation next turn
                } else if *until <= now {
                    instruction_queue.add(QueuedInstruction::Finished);
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
                commands.entity(actor).insert(Healing {
                    amount: healing_amount as u16,
                });

                if *until <= now {
                    instruction_queue.add(QueuedInstruction::Finished);
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

    #[allow(clippy::too_many_lines)]
    fn plan(
        &mut self,
        envir: &Envir,
        pos: Pos,
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
            (Self::ExaminingPos(curr), QueuedInstruction::Offset(direction)) => {
                let nbor = direction.to_nbor();
                self.handle_offset(envir.get_nbor(*curr, nbor), nbor)
            }
            (
                _,
                QueuedInstruction::CancelAction
                | QueuedInstruction::Wait
                | QueuedInstruction::Sleep,
            )
            | (Self::Attacking, QueuedInstruction::Attack)
            | (Self::Smashing, QueuedInstruction::Smash)
            | (Self::ExaminingPos(_), QueuedInstruction::ExaminePos)
            | (Self::ExaminingZoneLevel(_), QueuedInstruction::ExamineZoneLevel) => {
                *self = Self::Normal;
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::Offset(direction)) => {
                let nbor = direction.to_nbor();
                self.handle_offset(envir.get_nbor(pos, nbor), nbor)
            }
            (_, QueuedInstruction::Wield(entity)) => {
                PlayerBehavior::Perform(PlannedAction::Wield { entity })
            }
            (_, QueuedInstruction::Unwield(entity)) => {
                PlayerBehavior::Perform(PlannedAction::Unwield { entity })
            }
            (_, QueuedInstruction::Pickup(entity)) => {
                PlayerBehavior::Perform(PlannedAction::Pickup { entity })
            }
            (_, QueuedInstruction::Dump(entity, direction)) => {
                PlayerBehavior::Perform(PlannedAction::Dump { entity, direction })
            }
            (_, QueuedInstruction::Attack) => self.handle_attack(envir, pos),
            (_, QueuedInstruction::Smash) => self.handle_smash(envir, pos),
            (_, QueuedInstruction::Close) => self.handle_close(envir, pos),
            (_, QueuedInstruction::ExamineItem(entity)) => {
                PlayerBehavior::Perform(PlannedAction::ExamineItem { entity })
            }
            (_, QueuedInstruction::ExaminePos) => {
                let pos = Pos::from(&Focus::new(self, pos));
                *self = Self::ExaminingPos(pos);
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::ExamineZoneLevel) => {
                let target = ZoneLevel::from(&Focus::new(self, pos));
                *self = Self::ExaminingZoneLevel(target);
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::ChangePace) => {
                PlayerBehavior::Perform(PlannedAction::ChangePace)
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
            (_, QueuedInstruction::Interrupted) => {
                *self = Self::Normal;
                PlayerBehavior::Feedback(Message::error(Phrase::new(
                    "Iterrupted while not waiting",
                )))
            }
            (_, QueuedInstruction::Finished) => {
                *self = Self::Normal;
                PlayerBehavior::Feedback(Message::error(Phrase::new("Finished while not waiting")))
            }
        }
    }

    fn handle_offset(&mut self, target: Result<Pos, Message>, nbor: Nbor) -> PlayerBehavior {
        match (&self, target) {
            (Self::Sleeping { .. }, target) => {
                panic!("{:?} {:?}", &self, target);
            }
            (Self::ExaminingZoneLevel(current), _) => {
                let target = current.nbor(nbor);
                if let Some(target) = target {
                    *self = Self::ExaminingZoneLevel(target);
                    PlayerBehavior::NoEffect
                } else {
                    PlayerBehavior::Feedback(Message::warn(Phrase::new(
                        "invalid zone level to examine",
                    )))
                }
            }
            (Self::ExaminingPos(current), target) => {
                if let Some(target) = target.ok().or_else(|| current.raw_nbor(nbor)) {
                    *self = Self::ExaminingPos(target);
                    PlayerBehavior::NoEffect
                } else {
                    PlayerBehavior::Feedback(Message::warn(Phrase::new(
                        "invalid position to examine",
                    )))
                }
            }
            (Self::Normal, Ok(to)) => PlayerBehavior::Perform(PlannedAction::Step { to }),
            (Self::Attacking, Ok(target)) => {
                *self = Self::Normal;
                PlayerBehavior::Perform(PlannedAction::Attack { target })
            }
            (Self::Smashing, Ok(target)) => {
                *self = Self::Normal;
                PlayerBehavior::Perform(PlannedAction::Smash { target })
            }
            (Self::Closing, Ok(target)) => {
                *self = Self::Normal;
                PlayerBehavior::Perform(PlannedAction::Close { target })
            }
            (Self::Waiting { .. }, _) => {
                *self = Self::Normal;
                PlayerBehavior::NoEffect
            }
            (_, Err(message)) => PlayerBehavior::Feedback(message),
        }
    }

    fn handle_attack(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let attackable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Attack)
            .collect::<Vec<Pos>>();
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
            .collect::<Vec<Pos>>();
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

    fn handle_close(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let closable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Close)
            .collect::<Vec<Pos>>();
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
            Self::Waiting { .. } | Self::Sleeping { .. } | Self::Smashing | Self::Attacking => {
                WARN_TEXT_COLOR
            }
        }
    }
}

impl fmt::Display for PlayerActionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Normal => "",
            Self::Attacking => "Attacking",
            Self::Smashing => "Smashing",
            Self::Closing => "Closing",
            Self::Waiting { .. } => "Waiting",
            Self::Sleeping { .. } => "Sleeping",
            Self::ExaminingPos(_) => "Examining",
            Self::ExaminingZoneLevel(_) => "Examining map",
        })
    }
}
