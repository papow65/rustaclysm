use bevy::prelude::Component;

#[derive(Debug, Component)]
#[component(immutable)]
pub(super) struct LoadButtonArea;

#[derive(Debug, Component)]
#[component(immutable)]
pub(super) struct LogMessageWrapper;

#[derive(Debug, Component)]
#[component(immutable)]
pub(super) struct LogMessageField;
