use crate::prelude::*;
use bevy::prelude::{Color, Commands, NextState, Resource};
use std::fmt;

#[derive(Debug)]
enum PlayerBehavior {
    Perform(Action),
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
    Waiting(Milliseconds),
    Sleeping(Milliseconds),
    ExaminingPos(Pos),
    ExaminingZoneLevel(ZoneLevel),
}

impl PlayerActionState {
    pub(crate) fn plan_action(
        &mut self,
        commands: &mut Commands,
        next_gameplay_state: &mut NextState<GameplayScreenState>,
        envir: &mut Envir,
        instruction_queue: &mut InstructionQueue,
        pos: Pos,
        now: Milliseconds,
        enemies: &[Pos],
    ) -> Option<Action> {
        while let Some(instruction) = instruction_queue.pop() {
            match self.plan(next_gameplay_state, envir, pos, instruction, now) {
                PlayerBehavior::Perform(action) => {
                    return Some(action);
                }
                PlayerBehavior::Feedback(message) => {
                    commands.spawn(message);
                    // invalid instruction -> next instruction
                }
                PlayerBehavior::NoEffect => {
                    // valid instruction, but no action performed -> next instruction
                }
            }
        }

        match self {
            PlayerActionState::Waiting(until) => {
                if !enemies.is_empty() {
                    instruction_queue.add(QueuedInstruction::Interrupted);
                    None // process the cancellation next turn
                } else if *until <= now {
                    instruction_queue.add(QueuedInstruction::Finished);
                    None // process the cancellation next turn
                } else {
                    Some(Action::Stay {
                        duration: StayDuration::Long,
                    })
                }
            }
            PlayerActionState::Sleeping(until) => {
                // TODO interrupt on taking damage
                if *until <= now {
                    instruction_queue.add(QueuedInstruction::Finished);
                    None // process the cancellation next turn
                } else {
                    Some(Action::Stay {
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
        next_gameplay_state: &mut NextState<GameplayScreenState>,
        envir: &Envir,
        pos: Pos,
        instruction: QueuedInstruction,
        now: Milliseconds,
    ) -> PlayerBehavior {
        //println!("processing instruction: {instruction:?}");
        match (&self, instruction) {
            (PlayerActionState::Sleeping(_), QueuedInstruction::Finished) => {
                *self = PlayerActionState::Normal;
                PlayerBehavior::Feedback(Message::info().str("You wake up"))
            }
            (PlayerActionState::Sleeping(_), _) => {
                // Can not be interrupted
                PlayerBehavior::Feedback(Message::warn().str("You are still asleep. Zzz..."))
            }
            (PlayerActionState::Normal, QueuedInstruction::Offset(Direction::Here)) => {
                PlayerBehavior::Perform(Action::Stay {
                    duration: StayDuration::Short,
                })
            }
            (PlayerActionState::Normal, QueuedInstruction::Wait) => {
                *self = PlayerActionState::Waiting(now + Milliseconds::MINUTE);
                PlayerBehavior::Feedback(Message::info().str("Started waiting..."))
            }
            (PlayerActionState::Normal, QueuedInstruction::Sleep) => {
                *self = PlayerActionState::Sleeping(now + Milliseconds::EIGHT_HOURS);
                PlayerBehavior::Feedback(Message::info().str("Started sleeping... Zzz..."))
            }
            (PlayerActionState::Attacking, QueuedInstruction::Offset(Direction::Here)) => {
                PlayerBehavior::Feedback(Message::warn().str("You can't attack yourself"))
            }
            (PlayerActionState::ExaminingPos(curr), QueuedInstruction::Offset(direction)) => {
                let nbor = direction.to_nbor();
                self.handle_offset(envir.get_nbor(*curr, &nbor), &nbor)
            }
            (PlayerActionState::Normal, QueuedInstruction::Cancel) => {
                next_gameplay_state.set(GameplayScreenState::Menu);
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::Cancel | QueuedInstruction::Wait | QueuedInstruction::Sleep)
            | (PlayerActionState::Attacking, QueuedInstruction::Attack)
            | (PlayerActionState::Smashing, QueuedInstruction::Smash)
            | (PlayerActionState::ExaminingPos(_), QueuedInstruction::ExaminePos)
            | (PlayerActionState::ExaminingZoneLevel(_), QueuedInstruction::ExamineZoneLevel) => {
                *self = PlayerActionState::Normal;
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::Offset(direction)) => {
                let nbor = direction.to_nbor();
                self.handle_offset(envir.get_nbor(pos, &nbor), &nbor)
            }
            (_, QueuedInstruction::Wield) => PlayerBehavior::Perform(Action::Wield),
            (_, QueuedInstruction::Pickup) => PlayerBehavior::Perform(Action::Pickup),
            (_, QueuedInstruction::Dump) => PlayerBehavior::Perform(Action::Dump),
            (_, QueuedInstruction::Attack) => self.handle_attack(envir, pos),
            (_, QueuedInstruction::Smash) => self.handle_smash(envir, pos),
            (_, QueuedInstruction::Close) => self.handle_close(envir, pos),
            (_, QueuedInstruction::ExaminePos) => {
                let pos = Pos::from(&Focus::new(self, pos));
                *self = PlayerActionState::ExaminingPos(pos);
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::ExamineZoneLevel) => {
                let target = ZoneLevel::from(&Focus::new(self, pos));
                *self = PlayerActionState::ExaminingZoneLevel(target);
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::SwitchRunning) => PlayerBehavior::Perform(Action::SwitchRunning),
            (PlayerActionState::Waiting(_), QueuedInstruction::Interrupted) => {
                *self = PlayerActionState::Normal;
                PlayerBehavior::Feedback(Message::warn().str("You spot an enemy and stop waiting"))
            }
            (PlayerActionState::Waiting(_), QueuedInstruction::Finished) => {
                *self = PlayerActionState::Normal;
                PlayerBehavior::Feedback(Message::info().str("Finished waiting"))
            }
            (_, QueuedInstruction::Interrupted) => {
                *self = PlayerActionState::Normal;
                PlayerBehavior::Feedback(Message::error().str("Iterrupted while not waiting"))
            }
            (_, QueuedInstruction::Finished) => {
                *self = PlayerActionState::Normal;
                PlayerBehavior::Feedback(Message::error().str("Finished while not waiting"))
            }
        }
    }

    fn handle_offset(&mut self, target: Result<Pos, Message>, nbor: &Nbor) -> PlayerBehavior {
        match (&self, target) {
            (PlayerActionState::Sleeping(_), target) => {
                panic!("{:?} {:?}", &self, target);
            }
            (PlayerActionState::ExaminingZoneLevel(current), _) => {
                let target = current.nbor(nbor);
                if let Some(target) = target {
                    *self = PlayerActionState::ExaminingZoneLevel(target);
                    PlayerBehavior::NoEffect
                } else {
                    PlayerBehavior::Feedback(Message::warn().str("invalid zone level to examine"))
                }
            }
            (PlayerActionState::ExaminingPos(current), target) => {
                if let Some(target) = target.ok().or_else(|| current.raw_nbor(nbor)) {
                    *self = PlayerActionState::ExaminingPos(target);
                    PlayerBehavior::NoEffect
                } else {
                    PlayerBehavior::Feedback(Message::warn().str("invalid position to examine"))
                }
            }
            (PlayerActionState::Normal, Ok(target)) => {
                PlayerBehavior::Perform(Action::Step { target })
            }
            (PlayerActionState::Attacking, Ok(target)) => {
                *self = PlayerActionState::Normal;
                PlayerBehavior::Perform(Action::Attack { target })
            }
            (PlayerActionState::Smashing, Ok(target)) => {
                *self = PlayerActionState::Normal;
                PlayerBehavior::Perform(Action::Smash { target })
            }
            (PlayerActionState::Closing, Ok(target)) => {
                *self = PlayerActionState::Normal;
                PlayerBehavior::Perform(Action::Close { target })
            }
            (PlayerActionState::Waiting(_), _) => {
                *self = PlayerActionState::Normal;
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
            0 => PlayerBehavior::Feedback(Message::warn().str("no targets nearby")),
            1 => PlayerBehavior::Perform(Action::Attack {
                target: attackable_nbors[0],
            }),
            _ => {
                *self = PlayerActionState::Attacking;
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_smash(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let smashable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Smash)
            .collect::<Vec<Pos>>();
        match smashable_nbors.len() {
            0 => PlayerBehavior::Feedback(Message::warn().str("no targets nearby")),
            1 => PlayerBehavior::Perform(Action::Smash {
                target: smashable_nbors[0],
            }),
            _ => {
                *self = PlayerActionState::Smashing;
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_close(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let closable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Close)
            .collect::<Vec<Pos>>();
        match closable_nbors.len() {
            0 => PlayerBehavior::Feedback(Message::warn().str("nothing to close nearby")),
            1 => PlayerBehavior::Perform(Action::Close {
                target: closable_nbors[0],
            }),
            _ => {
                *self = PlayerActionState::Closing;
                PlayerBehavior::NoEffect
            }
        }
    }

    pub(crate) fn color(&self) -> Color {
        match self {
            Self::Normal | Self::Closing | Self::ExaminingPos(_) | Self::ExaminingZoneLevel(_) => {
                DEFAULT_TEXT_COLOR
            }
            Self::Waiting(_) | Self::Sleeping(_) | Self::Smashing | Self::Attacking => {
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
            Self::Waiting(_) => "Waiting",
            Self::Sleeping(_) => "Sleeping",
            Self::ExaminingPos(_) => "Examining",
            Self::ExaminingZoneLevel(_) => "Examining map",
        })
    }
}
