use bevy::prelude::Component;
use std::path::PathBuf;

#[derive(Component)]
pub(crate) struct Background;

#[derive(Component)]
pub(crate) struct LoadButtonArea;

#[derive(Component, Debug)]
pub(crate) struct LoadButton {
    pub(crate) path: PathBuf,
}

#[derive(Component)]
pub(crate) struct QuitButton;

#[derive(Component)]
pub(crate) struct MessageWrapper;

#[derive(Component)]
pub(crate) struct MessageField;
