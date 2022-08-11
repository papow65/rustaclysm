use crate::prelude::{Action, Direction, Envir, Instruction, Message, Pos, ZoneLevel};
use bevy::ecs::component::Component;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlayerActionState {
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
pub struct Player {
    pub state: PlayerActionState,
    pub camera_distance: f32,
}

impl Player {
    pub fn behave(
        &mut self,
        envir: &Envir,
        pos: Pos,
        instruction: Instruction,
    ) -> Result<Action, Option<Message>> {
        println!("processing instruction: {instruction:?}");

        match (self.state, instruction) {
            (PlayerActionState::Normal, Instruction::Offset(Direction::Here)) => Ok(Action::Stay),
            (PlayerActionState::Attacking, Instruction::Offset(Direction::Here)) => {
                Err(Some(Message::new("can't attack self")))
            }
            (PlayerActionState::ExaminingPos(curr), Instruction::Offset(direction)) => {
                self.handle_offset(curr, direction)
            }
            (PlayerActionState::Normal, Instruction::Cancel) => {
                Err(Some(Message::new("Press ctrl+c/d/q to exit")))
            }
            (_, Instruction::Cancel)
            | (PlayerActionState::Attacking, Instruction::Attack)
            | (PlayerActionState::Smashing, Instruction::Smash)
            | (PlayerActionState::ExaminingPos(_), Instruction::ExaminePos)
            | (PlayerActionState::ExaminingZoneLevel(_), Instruction::ExamineZoneLevel) => {
                self.state = PlayerActionState::Normal;
                Err(None)
            }
            (_, Instruction::Offset(offset)) => self.handle_offset(pos, offset),
            (_, Instruction::Pickup) => Ok(Action::Pickup),
            (_, Instruction::Dump) => Ok(Action::Dump),
            (_, Instruction::Attack) => self.handle_attack(envir, pos),
            (_, Instruction::Smash) => self.handle_smash(envir, pos),
            (_, Instruction::ExaminePos) => {
                self.state = PlayerActionState::ExaminingPos(pos);
                Ok(Action::ExaminePos { target: pos })
            }
            (_, Instruction::ExamineZoneLevel) => {
                let target = ZoneLevel::from(pos);
                self.state = PlayerActionState::ExaminingZoneLevel(target);
                Ok(Action::ExamineZoneLevel { target })
            }
            (_, Instruction::SwitchRunning) => Ok(Action::SwitchRunning),
        }
    }

    fn handle_offset(
        &mut self,
        reference: Pos,
        direction: Direction,
    ) -> Result<Action, Option<Message>> {
        if let PlayerActionState::ExaminingZoneLevel(current) = self.state {
            let Pos { x, level, z } = direction.get_relative_pos();
            let target = current.offset(ZoneLevel { x, level, z });
            if let Some(target) = target {
                self.state = PlayerActionState::ExaminingZoneLevel(target);
                return Ok(Action::ExamineZoneLevel { target });
            } else {
                return Err(Some(Message::new("invalid target")));
            }
        }

        let target = reference.offset(direction.get_relative_pos());
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
            Err(Some(Message::new(
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
            .nbors_for_exploring(pos, Instruction::Attack)
            .collect::<Vec<Pos>>();
        match attackable_nbors.len() {
            0 => Err(Some(Message::new("no targets nearby"))),
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
            .nbors_for_exploring(pos, Instruction::Smash)
            .collect::<Vec<Pos>>();
        match smashable_nbors.len() {
            0 => Err(Some(Message::new("no targets nearby"))),
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
