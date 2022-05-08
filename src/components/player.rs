use bevy::ecs::component::Component;
use std::fmt::{Display, Formatter};

use crate::components::{Action, Instruction, Message, Pos};
use crate::resources::Envir;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlayerActionState {
    Normal,
    Attacking,
    Smashing,
    Examining(Pos),
}

impl Display for PlayerActionState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PlayerActionState::Normal => "",
                PlayerActionState::Attacking => "Attacking",
                PlayerActionState::Smashing => "Smashing",
                PlayerActionState::Examining(_) => "Examining",
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
            (PlayerActionState::Normal, Instruction::Offset(Pos(0, 0, 0))) => Ok(Action::Stay),
            (PlayerActionState::Attacking, Instruction::Offset(Pos(0, 0, 0))) => {
                Err(Some(Message::new("can't attack self")))
            }
            (PlayerActionState::Examining(curr), Instruction::Offset(offset)) => {
                self.handle_offset(curr, offset)
            }
            (_, Instruction::Offset(offset)) => self.handle_offset(pos, offset),
            (_, Instruction::Pickup) => Ok(Action::Pickup),
            (_, Instruction::Dump) => Ok(Action::Dump),
            (_, Instruction::Attack) => self.handle_attack(envir, pos),
            (_, Instruction::Smash) => self.handle_smash(envir, pos),
            (PlayerActionState::Normal, Instruction::Cancel) => {
                Err(Some(Message::new("Press ctrl+c/d/q to exit")))
            }
            (_, Instruction::Cancel)
            | (PlayerActionState::Examining(_), Instruction::SwitchExamining) => {
                self.state = PlayerActionState::Normal;
                Err(None)
            }
            (_, Instruction::SwitchExamining) => {
                self.state = PlayerActionState::Examining(pos);
                Ok(Action::Examine { target: pos })
            }
            (_, Instruction::SwitchRunning) => Ok(Action::SwitchRunning),
        }
    }

    fn handle_offset(&mut self, reference: Pos, offset: Pos) -> Result<Action, Option<Message>> {
        let target = reference.offset(offset);
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
                PlayerActionState::Examining(_) => {
                    self.state = PlayerActionState::Examining(target);
                    Action::Examine { target }
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
                Err(Some(Message::new("attacking...")))
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
                Err(Some(Message::new("smashing...")))
            }
        }
    }
}
