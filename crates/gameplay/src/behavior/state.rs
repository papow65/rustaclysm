use crate::{Interruption, QueuedInstruction};
use bevy::prelude::{Resource, debug, warn};

/// Not a bevy state, because we don't use bevy state mechanisms.
#[derive(Debug, Default)]
pub(crate) enum PrivateBehaviorState {
    BehaviorLoop {
        /// Can be empty
        queue: Vec<QueuedInstruction>,
    },
    #[default]
    WaitingForUser,
}

/// Not a bevy state, because we don't use bevy state mechanisms.
#[derive(Debug, Default, Resource)]
pub(crate) struct BehaviorState(PrivateBehaviorState);

impl BehaviorState {
    const fn empty_behavior_loop() -> PrivateBehaviorState {
        PrivateBehaviorState::BehaviorLoop { queue: Vec::new() }
    }

    pub(crate) fn add(&mut self, instruction: QueuedInstruction) {
        let queue = match &mut self.0 {
            PrivateBehaviorState::BehaviorLoop { queue } => queue,
            PrivateBehaviorState::WaitingForUser => {
                self.0 = Self::empty_behavior_loop();
                let PrivateBehaviorState::BehaviorLoop { queue } = &mut self.0 else {
                    unreachable!();
                };
                queue
            }
        };

        // Wait for an instruction to be processed until adding a duplicate when holding a key down.
        if !instruction.held_key_allowed() || !queue.contains(&instruction) {
            queue.insert(0, instruction);
        }
    }

    pub(crate) fn interrupt(&mut self, interruption: Interruption) {
        self.add(QueuedInstruction::Interrupt(interruption));
    }

    pub(crate) fn pop(&mut self) -> Option<QueuedInstruction> {
        match &mut self.0 {
            PrivateBehaviorState::BehaviorLoop { queue } => queue.pop(),
            PrivateBehaviorState::WaitingForUser => None,
        }
    }

    pub(crate) fn log_if_long(&self) {
        if let PrivateBehaviorState::BehaviorLoop { queue } = &self.0
            && 1 < queue.len()
        {
            warn!("Unprocessed key codes: {:?}", queue);
        }
    }

    pub(super) fn start_waiting(&mut self) {
        // The queue may not be empty if an automatic action added an interrupt.
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
        matches!(self.0, PrivateBehaviorState::BehaviorLoop { .. })
    }
}
