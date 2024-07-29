use bevy::prelude::Component;
use std::path::PathBuf;

#[derive(Component)]
pub(super) struct Background;

#[derive(Component)]
pub(super) struct LoadButtonArea;

#[derive(Component, Debug)]
pub(super) struct LoadButton {
    pub(super) path: PathBuf,
}

#[derive(Component)]
pub(super) struct MessageWrapper;

#[derive(Component)]
pub(super) struct MessageField;
