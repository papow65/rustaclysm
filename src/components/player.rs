use bevy::ecs::component::Component;

use super::super::components::{Action, Instruction, Message, Pos};
use super::super::resources::Envir;

#[derive(Component)]
pub struct Player {
    pub camera_distance: f32,
}

impl Player {
    pub fn behave(
        combo: &mut Option<Instruction>,
        envir: &Envir,
        pos: Pos,
        instruction: Instruction,
    ) -> Result<Action, Vec<(Message,)>> {
        println!("processing instruction: {instruction:?}");

        let mut action_in_progress = combo.take();
        let mut target = pos;

        match instruction {
            Instruction::Offset(Pos(0, 0, 0)) => {
                if action_in_progress == Some(Instruction::Attack) {
                    return Err(vec![Message::new("can't attack self".to_string())]);
                } else {
                    return Ok(Action::Stay);
                }
            }
            Instruction::Offset(offset) => {
                target.0 += offset.0;
                target.1 += offset.1;
                target.2 += offset.2;
            }
            Instruction::Pickup => {
                return Ok(Action::Pickup);
            }
            Instruction::Dump => {
                return Ok(Action::Dump);
            }
            Instruction::Attack => {
                let attackable_nbors = envir
                    .nbors_for_exploring(pos, instruction)
                    .collect::<Vec<Pos>>();
                match attackable_nbors.len() {
                    0 => {
                        return Err(vec![Message::new("no targets nearby".to_string())]);
                    }
                    1 => {
                        action_in_progress = Some(Instruction::Attack);
                        target = attackable_nbors[0];
                    }
                    _ => {
                        combo.replace(instruction);
                        return Err(vec![Message::new("attacking...".to_string())]);
                    }
                }
            }
            Instruction::Smash => {
                let smashable_nbors = envir
                    .nbors_for_exploring(pos, instruction)
                    .collect::<Vec<Pos>>();
                match smashable_nbors.len() {
                    0 => {
                        return Err(vec![Message::new("no targets nearby".to_string())]);
                    }
                    1 => {
                        action_in_progress = Some(Instruction::Smash);
                        target = smashable_nbors[0];
                    }
                    _ => {
                        combo.replace(instruction);
                        return Err(vec![Message::new("smashing...".to_string())]);
                    }
                }
            }
            Instruction::SwitchRunning => {
                return Ok(Action::SwitchRunning);
            }
        }

        if target.in_bounds() {
            Ok(match action_in_progress {
                Some(Instruction::Attack) => Action::Attack { target },
                Some(Instruction::Smash) => Action::Smash { target },
                None => Action::Step { target },
                _ => panic!(),
            })
        } else {
            Err(vec![Message::new(
                if action_in_progress.is_some() {
                    "invalid target"
                } else {
                    "you can't leave"
                }
                .to_string(),
            )])
        }
    }
}
