use bevy::prelude::KeyCode;
use keyboard::Key;
use strum::VariantArray;

#[derive(Clone, Copy, Debug, VariantArray)]
pub enum SelectionListStep {
    ManyUp,
    SingleUp,
    SingleDown,
    ManyDown,
}

impl SelectionListStep {
    pub(crate) const fn amount(self) -> usize {
        if matches!(self, Self::ManyUp | Self::ManyDown) {
            10
        } else {
            1
        }
    }

    pub(crate) const fn is_backwards(self) -> bool {
        matches!(self, Self::ManyUp | Self::SingleUp)
    }
}

impl From<SelectionListStep> for Key {
    fn from(step: SelectionListStep) -> Self {
        Self::Code(match step {
            SelectionListStep::ManyUp => KeyCode::PageUp,
            SelectionListStep::SingleUp => KeyCode::ArrowUp,
            SelectionListStep::SingleDown => KeyCode::ArrowDown,
            SelectionListStep::ManyDown => KeyCode::PageDown,
        })
    }
}
