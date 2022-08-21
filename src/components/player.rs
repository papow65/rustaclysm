use crate::prelude::{Action, Direction, Envir, Level, Message, Pos, QueuedInstruction, ZoneLevel};
use bevy::ecs::component::Component;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum PlayerActionState {
    Normal,
    Attacking,
    Smashing,
    ExaminingPos(Pos),
    ExaminingZoneLevel(ZoneLevel),
}

impl Display for PlayerActionState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Normal => "",
                Self::Attacking => "Attacking",
                Self::Smashing => "Smashing",
                Self::ExaminingPos(_) => "Examining",
                Self::ExaminingZoneLevel(_) => "Examining map",
            }
        )
        .unwrap();

        Ok(())
    }
}

#[derive(Component)]
pub(crate) struct Player {
    pub(crate) state: PlayerActionState,
    pub(crate) camera_distance: f32,
}

impl Player {
    pub(crate) fn is_shown(&self, level: Level, player_pos: Pos) -> bool {
        let reference = match self.state {
            PlayerActionState::ExaminingPos(pos) => pos,
            PlayerActionState::ExaminingZoneLevel(zone_level) => zone_level.base_pos(),
            _ => player_pos,
        };

        level.visible_from(reference.level)
    }

    pub(crate) fn behave(
        &mut self,
        envir: &Envir,
        pos: Pos,
        instruction: QueuedInstruction,
    ) -> Result<Action, Option<Message>> {
        println!("processing instruction: {instruction:?}");

        match (self.state, instruction) {
            (PlayerActionState::Normal, QueuedInstruction::Offset(Direction::Here)) => {
                Ok(Action::Stay)
            }
            (PlayerActionState::Attacking, QueuedInstruction::Offset(Direction::Here)) => {
                Err(Some(Message::warn("can't attack self")))
            }
            (PlayerActionState::ExaminingPos(curr), QueuedInstruction::Offset(direction)) => {
                self.handle_offset(envir, curr, direction)
            }
            (PlayerActionState::Normal, QueuedInstruction::Cancel) => {
                Err(Some(Message::warn("Press ctrl+c/d/q to exit")))
            }
            (_, QueuedInstruction::Cancel)
            | (PlayerActionState::Attacking, QueuedInstruction::Attack)
            | (PlayerActionState::Smashing, QueuedInstruction::Smash)
            | (PlayerActionState::ExaminingPos(_), QueuedInstruction::ExaminePos)
            | (PlayerActionState::ExaminingZoneLevel(_), QueuedInstruction::ExamineZoneLevel) => {
                self.state = PlayerActionState::Normal;
                Err(None)
            }
            (_, QueuedInstruction::Offset(offset)) => self.handle_offset(envir, pos, offset),
            (_, QueuedInstruction::Pickup) => Ok(Action::Pickup),
            (_, QueuedInstruction::Dump) => Ok(Action::Dump),
            (_, QueuedInstruction::Attack) => self.handle_attack(envir, pos),
            (_, QueuedInstruction::Smash) => self.handle_smash(envir, pos),
            (_, QueuedInstruction::ExaminePos) => {
                self.state = PlayerActionState::ExaminingPos(pos);
                Ok(Action::ExaminePos { target: pos })
            }
            (_, QueuedInstruction::ExamineZoneLevel) => {
                let target = ZoneLevel::from(pos);
                self.state = PlayerActionState::ExaminingZoneLevel(target);
                Ok(Action::ExamineZoneLevel { target })
            }
            (_, QueuedInstruction::SwitchRunning) => Ok(Action::SwitchRunning),
        }
    }

    fn handle_offset(
        &mut self,
        envir: &Envir,
        reference: Pos,
        direction: Direction,
    ) -> Result<Action, Option<Message>> {
        if let PlayerActionState::ExaminingZoneLevel(current) = self.state {
            let target = current.nbor(direction.to_nbor());
            if let Some(target) = target {
                self.state = PlayerActionState::ExaminingZoneLevel(target);
                return Ok(Action::ExamineZoneLevel { target });
            } else {
                return Err(Some(Message::warn("invalid zone level to examine")));
            }
        }

        let target = envir.get_nbor(reference, &direction.to_nbor());
        if let Some(target) = target {
            Ok(match self.state {
                PlayerActionState::Normal => Action::Step { target },
                PlayerActionState::Attacking => {
                    self.state = PlayerActionState::Normal;
                    Action::Attack { target }
                }
                PlayerActionState::Smashing => {
                    self.state = PlayerActionState::Normal;
                    Action::Smash { target }
                }
                PlayerActionState::ExaminingPos(current) => {
                    assert!(reference == current, "{reference:?} {current:?}");
                    self.state = PlayerActionState::ExaminingPos(target);
                    Action::ExaminePos { target }
                }
                PlayerActionState::ExaminingZoneLevel(_) => {
                    unreachable!();
                }
            })
        } else {
            Err(Some(Message::error(
                if self.state == PlayerActionState::Normal {
                    "you can't leave"
                } else {
                    "invalid target"
                },
            )))
        }
    }

    fn handle_attack(&mut self, envir: &Envir, pos: Pos) -> Result<Action, Option<Message>> {
        let attackable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Attack)
            .collect::<Vec<Pos>>();
        match attackable_nbors.len() {
            0 => Err(Some(Message::warn("no targets nearby"))),
            1 => Ok(Action::Attack {
                target: attackable_nbors[0],
            }),
            _ => {
                self.state = PlayerActionState::Attacking;
                Err(None)
            }
        }
    }

    fn handle_smash(&mut self, envir: &Envir, pos: Pos) -> Result<Action, Option<Message>> {
        let smashable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Smash)
            .collect::<Vec<Pos>>();
        match smashable_nbors.len() {
            0 => Err(Some(Message::warn("no targets nearby"))),
            1 => Ok(Action::Smash {
                target: smashable_nbors[0],
            }),
            _ => {
                self.state = PlayerActionState::Smashing;
                Err(None)
            }
        }
    }
}
