use bevy::prelude::{Handle, Image, Resource};

/// This prevents dropping the background image handle.
/// Preserving this handle ensures a smooth transition at the end of gameplay.
#[derive(Default, Resource)]
pub(crate) struct BackgroundHandle(pub Handle<Image>);
