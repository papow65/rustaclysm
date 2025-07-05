use bevy::prelude::{Component, TextSpan};
use std::num::Saturating;

#[derive(Component)]
#[component(immutable)]
pub(super) struct FpsText;

#[derive(Component)]
#[component(immutable)]
pub(super) struct TimeText;

#[derive(Component)]
#[component(immutable)]
pub(super) struct HealthText;

#[derive(Component)]
#[component(immutable)]
pub(super) struct StaminaText;

#[derive(Component)]
#[component(immutable)]
pub(super) struct BreathText;

#[derive(Component)]
#[component(immutable)]
pub(super) struct WalkingModeTextSpan;

#[derive(Component)]
#[component(immutable)]
pub(super) struct SpeedTextSpan;

#[derive(Component)]
#[component(immutable)]
pub(super) struct PlayerActionStateText;

#[derive(Component)]
#[component(immutable)]
pub(super) struct WieldedText;

#[derive(Component)]
#[component(immutable)]
pub(super) struct EnemiesText;

#[derive(Component)]
#[component(immutable)]
pub(super) struct DetailsText;

#[derive(Component)]
#[component(immutable)]
pub(super) struct LogDisplay;

#[derive(Component)]
#[component(immutable)]
pub(super) struct LastMessage;

#[derive(Clone, Debug, Component)]
pub(super) struct LastMessageCount(Saturating<u16>);

impl LastMessageCount {
    pub(crate) const fn is_single(&self) -> bool {
        self.0.0 == 1
    }

    pub(crate) fn raise(&mut self) {
        self.0 += 1;
    }

    pub(crate) fn text(&self) -> TextSpan {
        TextSpan::from(format!(" ({}x)", self.0))
    }
}

impl Default for LastMessageCount {
    fn default() -> Self {
        Self(Saturating(1))
    }
}

#[derive(Component)]
#[component(immutable)]
pub(super) struct TransientMessage;
