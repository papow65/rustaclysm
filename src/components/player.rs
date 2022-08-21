use crate::prelude::{
    Action, Direction, Envir, Level, Message, Nbor, Pos, QueuedInstruction, ZoneLevel,
};
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
                let nbor = direction.to_nbor();
                self.handle_offset(envir.get_nbor(curr, &nbor), &nbor)
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
            (_, QueuedInstruction::Offset(direction)) => {
                let nbor = direction.to_nbor();
                self.handle_offset(envir.get_nbor(pos, &nbor), &nbor)
            }
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
        target: Result<Pos, Message>,
        nbor: &Nbor,
    ) -> Result<Action, Option<Message>> {
        match (self.state, target) {
            (PlayerActionState::ExaminingZoneLevel(current), _) => {
                let target = current.nbor(nbor);
                if let Some(target) = target {
                    self.state = PlayerActionState::ExaminingZoneLevel(target);
                    Ok(Action::ExamineZoneLevel { target })
                } else {
                    Err(Some(Message::warn("invalid zone level to examine")))
                }
            }
            (PlayerActionState::ExaminingPos(current), target) => {
                if let Some(target) = target.ok().or_else(|| current.raw_nbor(nbor)) {
                    self.state = PlayerActionState::ExaminingPos(target);
                    Ok(Action::ExaminePos { target })
                } else {
                    Err(Some(Message::warn("invalid position to examine")))
                }
            }
            (PlayerActionState::Normal, Ok(target)) => Ok(Action::Step { target }),
            (PlayerActionState::Attacking, Ok(target)) => {
                self.state = PlayerActionState::Normal;
                Ok(Action::Attack { target })
            }
            (PlayerActionState::Smashing, Ok(target)) => {
                self.state = PlayerActionState::Normal;
                Ok(Action::Smash { target })
            }
            (_, Err(message)) => Err(Some(message)),
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
