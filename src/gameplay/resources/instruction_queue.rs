use crate::prelude::QueuedInstruction;
use bevy::prelude::Resource;

#[derive(Debug, Default, Resource)]
pub(crate) struct InstructionQueue {
    queue: Vec<QueuedInstruction>,
    continuous: Vec<QueuedInstruction>,
    waiting_for_user: bool,
}

impl InstructionQueue {
    pub(crate) fn add(&mut self, instruction: QueuedInstruction) {
        // Wait for an instruction to be processed until adding a duplicate when holding a key down.
        if !self.continuous.contains(&instruction) || !self.queue.contains(&instruction) {
            self.queue.insert(0, instruction.clone());
            self.continuous.push(instruction);
        }

        self.waiting_for_user = false;
    }

    pub(crate) fn interrupt(&mut self, instruction: &QueuedInstruction) {
        self.continuous.retain(|k| k != instruction);
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
        assert!(self.queue.is_empty());
        assert!(!self.waiting_for_user);

        self.waiting_for_user = true;
    }

    pub(crate) fn is_waiting(&self) -> bool {
        self.waiting_for_user
    }
}
