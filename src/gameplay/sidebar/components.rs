use bevy::prelude::Component;

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
