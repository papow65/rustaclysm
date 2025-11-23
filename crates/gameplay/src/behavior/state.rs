use crate::{Interruption, QueuedInstruction};
use bevy::prelude::{Resource, debug, warn};

/// Not a bevy state, because we don't use bevy state mechanisms.
#[derive(Debug, Default)]
pub(crate) enum PrivateBehaviorState {
    BehaviorLoop {
        /// Can be empty
        player_instructions: Vec<QueuedInstruction>,
    },
    #[default]
    WaitingForUser,
}

/// Not a bevy state, because we don't use bevy state mechanisms.
#[derive(Debug, Default, Resource)]
pub(crate) struct BehaviorState(PrivateBehaviorState);

impl BehaviorState {
    const fn empty_behavior_loop() -> PrivateBehaviorState {
        PrivateBehaviorState::BehaviorLoop {
            player_instructions: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, instruction: QueuedInstruction) {
        let player_instructions = match &mut self.0 {
            PrivateBehaviorState::BehaviorLoop {
                player_instructions,
            } => player_instructions,
            PrivateBehaviorState::WaitingForUser => {
                self.0 = Self::empty_behavior_loop();
                let PrivateBehaviorState::BehaviorLoop {
                    player_instructions,
                } = &mut self.0
                else {
                    unreachable!();
                };
                player_instructions
            }
        };

        // Wait for an instruction to be processed until adding a duplicate when holding a key down.
        if !instruction.held_key_allowed() || !player_instructions.contains(&instruction) {
            player_instructions.insert(0, instruction);
        }
    }

    pub(crate) fn interrupt(&mut self, interruption: Interruption) {
        self.push(QueuedInstruction::Interrupt(interruption));
    }

    pub(crate) fn pop(&mut self) -> Option<QueuedInstruction> {
        match &mut self.0 {
            PrivateBehaviorState::BehaviorLoop {
                player_instructions,
            } => player_instructions.pop(),
            PrivateBehaviorState::WaitingForUser => None,
        }
    }

    pub(crate) fn log_if_long(&self) {
        if let PrivateBehaviorState::BehaviorLoop {
            player_instructions,
        } = &self.0
            && 1 < player_instructions.len()
        {
            warn!("Unprocessed key codes: {:?}", player_instructions);
        }
    }

    pub(crate) fn start_waiting(&mut self) {
        // `player_instructions` may not be empty if an automatic action added an interrupt.
        assert!(
            self.looping_behavior(),
            "Waiting for user input shouldn't already be in effect"
        );

        self.0 = PrivateBehaviorState::WaitingForUser;
        debug!("Waiting for user action");
    }

    pub(crate) fn stop_waiting(&mut self) {
        self.0 = Self::empty_behavior_loop();
    }

    pub(super) const fn looping_behavior(&self) -> bool {
        // `player_instructions` may be empty, because it may be an npc's turn
        matches!(self.0, PrivateBehaviorState::BehaviorLoop { .. })
    }
}
