use crate::prelude::*;
use bevy::prelude::{Commands, Component, NextState};
use std::{cmp::Ordering, fmt};

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
    Feedback(Message),
    NoEffect,
}

#[derive(Component)]
pub(crate) struct Player {
    pub(crate) state: PlayerActionState,
}

impl Player {
    pub(crate) fn plan_action(
        &mut self,
        commands: &mut Commands,
        next_state: &mut NextState<GameplayScreenState>,
        envir: &mut Envir,
        instruction_queue: &mut InstructionQueue,
        pos: Pos,
        now: Milliseconds,
    ) -> Option<Action> {
        loop {
            if let Some(instruction) = instruction_queue.pop() {
                match self.plan(next_state, envir, pos, instruction, now) {
                    PlayerBehavior::Perform(action) => break Some(action),
                    PlayerBehavior::Feedback(message) => {
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
        next_state: &mut NextState<GameplayScreenState>,
        envir: &Envir,
        pos: Pos,
        instruction: QueuedInstruction,
        now: Milliseconds,
    ) -> PlayerBehavior {
        //println!("processing instruction: {instruction:?}");
        match (self.state, instruction) {
            (PlayerActionState::Normal, QueuedInstruction::Offset(Direction::Here)) => {
                PlayerBehavior::Perform(Action::Stay)
            }
            (PlayerActionState::Normal, QueuedInstruction::Wait) => {
                self.state = PlayerActionState::Waiting(now + Milliseconds::MINUTE);
                PlayerBehavior::Feedback(Message::info("Started waiting..."))
            }
            (PlayerActionState::Attacking, QueuedInstruction::Offset(Direction::Here)) => {
                PlayerBehavior::Feedback(Message::warn("can't attack self"))
            }
            (PlayerActionState::ExaminingPos(curr), QueuedInstruction::Offset(direction)) => {
                let nbor = direction.to_nbor();
                self.handle_offset(envir.get_nbor(curr, &nbor), &nbor)
            }
            (PlayerActionState::Normal, QueuedInstruction::Cancel) => {
                next_state.set(GameplayScreenState::Menu);
                PlayerBehavior::NoEffect
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
            (_, QueuedInstruction::Wield) => PlayerBehavior::Perform(Action::Wield),
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
            (PlayerActionState::Waiting(_), QueuedInstruction::Interrupted) => {
                self.state = PlayerActionState::Normal;
                PlayerBehavior::Feedback(Message::warn("You see an enemy and stop waiting"))
            }
            (PlayerActionState::Waiting(_), QueuedInstruction::Finished) => {
                self.state = PlayerActionState::Normal;
                PlayerBehavior::Feedback(Message::info("Finished waiting"))
            }
            (_, QueuedInstruction::Interrupted) => {
                self.state = PlayerActionState::Normal;
                PlayerBehavior::Feedback(Message::error("Iterrupted while not waiting"))
            }
            (_, QueuedInstruction::Finished) => {
                self.state = PlayerActionState::Normal;
                PlayerBehavior::Feedback(Message::error("Finished while not waiting"))
            }
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
                    PlayerBehavior::Feedback(Message::warn("invalid zone level to examine"))
                }
            }
            (PlayerActionState::ExaminingPos(current), target) => {
                if let Some(target) = target.ok().or_else(|| current.raw_nbor(nbor)) {
                    self.state = PlayerActionState::ExaminingPos(target);
                    PlayerBehavior::NoEffect
                } else {
                    PlayerBehavior::Feedback(Message::warn("invalid position to examine"))
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
            (_, Err(message)) => PlayerBehavior::Feedback(message),
        }
    }

    fn handle_attack(&mut self, envir: &Envir, pos: Pos) -> PlayerBehavior {
        let attackable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Attack)
            .collect::<Vec<Pos>>();
        match attackable_nbors.len() {
            0 => PlayerBehavior::Feedback(Message::warn("no targets nearby")),
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
            0 => PlayerBehavior::Feedback(Message::warn("no targets nearby")),
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
            0 => PlayerBehavior::Feedback(Message::warn("nothing to close nearby")),
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

    pub(crate) fn is_pos_shown(
        &self,
        shown_pos: Pos,
        elevation_visibility: ElevationVisibility,
    ) -> bool {
        match self {
            Focus::Pos(focus_pos) => {
                shown_pos.level <= focus_pos.level
                    || (elevation_visibility == ElevationVisibility::Shown
                        && ((shown_pos.z
                            - focus_pos.z
                            - i32::from((shown_pos.level - focus_pos.level).h))
                            < (focus_pos.x - shown_pos.x).abs()))
            }
            Focus::ZoneLevel(zone_level) => {
                let focus_level = zone_level.level;
                match (focus_level.compare_to_ground(), elevation_visibility) {
                    (Ordering::Less, _) | (Ordering::Equal, ElevationVisibility::Shown) => {
                        // Below ground elevation is ignored, so only the current level is shown.
                        // And on ground, with elevation hidden, show only the current level.
                        shown_pos.level == focus_level
                    }
                    (Ordering::Equal | Ordering::Greater, ElevationVisibility::Hidden) => {
                        // On or above ground, with elevation shown, show everything on or above ground
                        Level::ZERO <= shown_pos.level
                    }
                    (Ordering::Greater, ElevationVisibility::Shown) => {
                        // Above ground, with elevation hidden, show everything between ground and focus
                        Level::ZERO <= shown_pos.level && shown_pos.level <= focus_level
                    }
                }
            }
        }
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self::Pos(Pos::ORIGIN)
    }
}

impl From<&Focus> for Level {
    fn from(focus: &Focus) -> Self {
        match focus {
            Focus::Pos(pos) => pos.level,
            Focus::ZoneLevel(zone_level) => zone_level.level,
        }
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
