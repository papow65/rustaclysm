use crate::prelude::{
    Action, Direction, Envir, InstructionQueue, Level, Message, Milliseconds, Nbor, Pos,
    QueuedInstruction, ZoneLevel,
};
use bevy::prelude::{Commands, Component};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum PlayerActionState {
    Normal,
    Attacking,
    Smashing,
    Closing,
    Waiting(Milliseconds),
    ExaminingPos(Pos),
    ExaminingZoneLevel(ZoneLevel),
}

impl fmt::Display for PlayerActionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Normal => "",
            Self::Attacking => "Attacking",
            Self::Smashing => "Smashing",
            Self::Closing => "Closing",
            Self::Waiting(_) => "Waiting",
            Self::ExaminingPos(_) => "Examining",
            Self::ExaminingZoneLevel(_) => "Examining map",
        })
    }
}

pub(crate) enum PlayerBehavior {
    Perform(Action),
    Warning(Message),
    NoEffect,
}

#[derive(Component)]
pub(crate) struct Player {
    pub(crate) state: PlayerActionState,
    pub(crate) camera_distance: f32,
}

impl Player {
    pub(crate) fn plan_action(
        &mut self,
        commands: &mut Commands,
        envir: &mut Envir,
        instruction_queue: &mut InstructionQueue,
        pos: Pos,
        now: Milliseconds,
    ) -> Option<Action> {
        loop {
            if let Some(instruction) = instruction_queue.pop() {
                match self.plan(envir, pos, instruction, now) {
                    PlayerBehavior::Perform(action) => break Some(action),
                    PlayerBehavior::Warning(message) => {
                        commands.spawn(message);
                        // invalid instruction -> next instruction
                    }
                    PlayerBehavior::NoEffect => {
                        // valid instruction, but no action performed -> next instruction
                    }
                }
            } else {
                break None;
            };
        }
    }

    fn plan(
        &mut self,
        envir: &Envir,
        pos: Pos,
        instruction: QueuedInstruction,
        now: Milliseconds,
    ) -> PlayerBehavior {
        println!("processing instruction: {instruction:?}");

        match (self.state, instruction) {
            (PlayerActionState::Normal, QueuedInstruction::Offset(Direction::Here)) => {
                PlayerBehavior::Perform(Action::Stay)
            }
            (PlayerActionState::Normal, QueuedInstruction::Wait) => {
                self.state = PlayerActionState::Waiting(now + Milliseconds::MINUTE);
                PlayerBehavior::Warning(Message::new("Started waiting..."))
            }
            (PlayerActionState::Attacking, QueuedInstruction::Offset(Direction::Here)) => {
                PlayerBehavior::Warning(Message::warn("can't attack self"))
            }
            (PlayerActionState::ExaminingPos(curr), QueuedInstruction::Offset(direction)) => {
                let nbor = direction.to_nbor();
                self.handle_offset(envir.get_nbor(curr, &nbor), &nbor)
            }
            (PlayerActionState::Normal, QueuedInstruction::Cancel) => {
                PlayerBehavior::Warning(Message::warn("Press ctrl+c/d/q to exit"))
            }
            (_, QueuedInstruction::Cancel | QueuedInstruction::Wait)
            | (PlayerActionState::Attacking, QueuedInstruction::Attack)
            | (PlayerActionState::Smashing, QueuedInstruction::Smash)
            | (PlayerActionState::ExaminingPos(_), QueuedInstruction::ExaminePos)
            | (PlayerActionState::ExaminingZoneLevel(_), QueuedInstruction::ExamineZoneLevel) => {
                self.state = PlayerActionState::Normal;
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::Offset(direction)) => {
                let nbor = direction.to_nbor();
                self.handle_offset(envir.get_nbor(pos, &nbor), &nbor)
            }
            (_, QueuedInstruction::Pickup) => PlayerBehavior::Perform(Action::Pickup),
            (_, QueuedInstruction::Dump) => PlayerBehavior::Perform(Action::Dump),
            (_, QueuedInstruction::Attack) => self.handle_attack(envir, pos),
            (_, QueuedInstruction::Smash) => self.handle_smash(envir, pos),
            (_, QueuedInstruction::Close) => self.handle_close(envir, pos),
            (_, QueuedInstruction::ExaminePos) => {
                let pos = Pos::from(&Focus::new(self, pos));
                self.state = PlayerActionState::ExaminingPos(pos);
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::ExamineZoneLevel) => {
                let target = ZoneLevel::from(&Focus::new(self, pos));
                self.state = PlayerActionState::ExaminingZoneLevel(target);
                PlayerBehavior::NoEffect
            }
            (_, QueuedInstruction::SwitchRunning) => PlayerBehavior::Perform(Action::SwitchRunning),
        }
    }

    fn handle_offset(&mut self, target: Result<Pos, Message>, nbor: &Nbor) -> PlayerBehavior {
        match (self.state, target) {
            (PlayerActionState::ExaminingZoneLevel(current), _) => {
                let target = current.nbor(nbor);
                if let Some(target) = target {
                    self.state = PlayerActionState::ExaminingZoneLevel(target);
                    PlayerBehavior::NoEffect
                } else {
                    PlayerBehavior::Warning(Message::warn("invalid zone level to examine"))
                }
            }
            (PlayerActionState::ExaminingPos(current), target) => {
                if let Some(target) = target.ok().or_else(|| current.raw_nbor(nbor)) {
                    self.state = PlayerActionState::ExaminingPos(target);
                    PlayerBehavior::NoEffect
                } else {
                    PlayerBehavior::Warning(Message::warn("invalid position to examine"))
                }
            }
            (PlayerActionState::Normal, Ok(target)) => {
                PlayerBehavior::Perform(Action::Step { target })
            }
            (PlayerActionState::Attacking, Ok(target)) => {
                self.state = PlayerActionState::Normal;
                PlayerBehavior::Perform(Action::Attack { target })
            }
            (PlayerActionState::Smashing, Ok(target)) => {
                self.state = PlayerActionState::Normal;
                PlayerBehavior::Perform(Action::Smash { target })
            }
            (PlayerActionState::Closing, Ok(target)) => {
                self.state = PlayerActionState::Normal;
                PlayerBehavior::Perform(Action::Close { target })
            }
            (PlayerActionState::Waiting(_), _) => {
                self.state = PlayerActionState::Normal;
                PlayerBehavior::NoEffect
            }
            (_, Err(message)) => PlayerBehavior::Warning(message),
        }
    }

    fn handle_attack(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let attackable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Attack)
            .collect::<Vec<Pos>>();
        match attackable_nbors.len() {
            0 => PlayerBehavior::Warning(Message::warn("no targets nearby")),
            1 => PlayerBehavior::Perform(Action::Attack {
                target: attackable_nbors[0],
            }),
            _ => {
                self.state = PlayerActionState::Attacking;
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_smash(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let smashable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Smash)
            .collect::<Vec<Pos>>();
        match smashable_nbors.len() {
            0 => PlayerBehavior::Warning(Message::warn("no targets nearby")),
            1 => PlayerBehavior::Perform(Action::Smash {
                target: smashable_nbors[0],
            }),
            _ => {
                self.state = PlayerActionState::Smashing;
                PlayerBehavior::NoEffect
            }
        }
    }

    fn handle_close(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let closable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Close)
            .collect::<Vec<Pos>>();
        match closable_nbors.len() {
            0 => PlayerBehavior::Warning(Message::warn("nothing to close nearby")),
            1 => PlayerBehavior::Perform(Action::Close {
                target: closable_nbors[0],
            }),
            _ => {
                self.state = PlayerActionState::Closing;
                PlayerBehavior::NoEffect
            }
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum Focus {
    Pos(Pos),
    ZoneLevel(ZoneLevel),
}

impl Focus {
    pub(crate) fn new(player: &Player, player_pos: Pos) -> Self {
        match player.state {
            PlayerActionState::ExaminingPos(target) => Focus::Pos(target),
            PlayerActionState::ExaminingZoneLevel(zone_level) => Focus::ZoneLevel(zone_level),
            _ => Focus::Pos(player_pos),
        }
    }

    pub(crate) fn is_shown(&self, level: Level) -> bool {
        let focus_level = match self {
            Focus::Pos(pos) => pos.level,
            Focus::ZoneLevel(zone_level) => zone_level.level,
        };
        level <= focus_level
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self::Pos(Pos::ORIGIN)
    }
}

impl From<&Focus> for Pos {
    fn from(focus: &Focus) -> Self {
        match focus {
            Focus::Pos(pos) => *pos,
            Focus::ZoneLevel(zone_level) => zone_level.base_pos(),
        }
    }
}

impl From<&Focus> for ZoneLevel {
    fn from(focus: &Focus) -> Self {
        match focus {
            Focus::Pos(pos) => ZoneLevel::from(*pos),
            Focus::ZoneLevel(zone_level) => *zone_level,
        }
    }
}
