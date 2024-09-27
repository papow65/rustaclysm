use bevy::prelude::Component;
use std::path::PathBuf;

#[derive(Debug, Component)]
pub(super) struct LoadButtonArea;

#[derive(Debug, Component)]
pub(super) struct LoadButton {
    pub(super) path: PathBuf,
}

#[derive(Debug, Component)]
pub(super) struct MessageWrapper;

#[derive(Debug, Component)]
pub(super) struct MessageField;

#[derive(Debug, Component)]
pub(super) struct QuitButton;
