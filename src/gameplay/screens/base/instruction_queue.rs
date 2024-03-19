use crate::prelude::{InputChange, Interruption, QueuedInstruction};
use bevy::prelude::Resource;

#[derive(Debug, Default, Resource)]
pub(crate) struct InstructionQueue {
    queue: Vec<QueuedInstruction>,
    waiting_for_user: bool,
}

impl InstructionQueue {
    pub(crate) fn add(&mut self, instruction: QueuedInstruction, change: InputChange) {
        // Wait for an instruction to be processed until adding a duplicate when holding a key down.
        if change == InputChange::JustPressed || !self.queue.contains(&instruction) {
            self.queue.insert(0, instruction);
        }

        self.waiting_for_user = false;
    }

    pub(crate) fn interrupt(&mut self, interruption: Interruption) {
        self.add(
            QueuedInstruction::Interrupt(interruption),
            InputChange::JustPressed,
        );
    }

    pub(crate) fn pop(&mut self) -> Option<QueuedInstruction> {
        self.queue.pop()
    }

    pub(crate) fn log_if_long(&self) {
        if 1 < self.queue.len() {
            println!("Unprocessed key codes: {:?}", self.queue);
        }
    }

    pub(crate) fn start_waiting(&mut self) {
        assert!(
            self.queue.is_empty(),
            "The player character must be present"
        );
        assert!(
            !self.waiting_for_user,
            "Waiting for user input shouldn't alrady be in effect"
        );

        self.waiting_for_user = true;
    }

    pub(crate) fn stop_waiting(&mut self) {
        self.waiting_for_user = false;
    }

    pub(crate) const fn is_waiting(&self) -> bool {
        self.waiting_for_user
    }
}
