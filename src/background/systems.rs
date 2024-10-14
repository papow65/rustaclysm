use crate::background::{component::Background, state::BackgroundState};
use crate::util::AssetPaths;
use bevy::prelude::{
    AssetServer, Commands, GlobalZIndex, ImageBundle, PositionType, Query, Res, StateScoped, Style,
    UiImage, Val, Window, With,
};

const BACKGROUND_WIDTH: f32 = 1522.0;
const BACKGROUND_HEIGHT: f32 = 1009.0;
const BACKGROUND_NAME: &str = "on_the_run.png";

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
    let background_scale = background_scale(windows.get_single().ok());
    let background_image = asset_server.load(AssetPaths::backgrounds().join(BACKGROUND_NAME));
    commands.spawn((
        ImageBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: background_scale.0,
                height: background_scale.1,
                ..Style::default()
            },
            image: UiImage {
                texture: background_image,
                ..UiImage::default()
            },
            ..ImageBundle::default()
        },
        GlobalZIndex(1),
        Background,
        StateScoped(BackgroundState),
    ));
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn resize_background(
    windows: Query<&Window>,
    mut backgrounds: Query<&mut Style, With<Background>>,
) {
    if let Ok(mut style) = backgrounds.get_single_mut() {
        (style.width, style.height) = background_scale(windows.get_single().ok());
    }
}

fn background_scale(window: Option<&Window>) -> (Val, Val) {
    let ratio = if let Some(window) = window {
        window.resolution.height() * BACKGROUND_WIDTH
            / (BACKGROUND_HEIGHT * window.resolution.width())
    } else {
        eprintln!("No window size available (yet?) to resize the background to");
        1.0
    };

    (
        Val::Percent(100.0 * ratio.max(1.0)),
        Val::Percent(100.0 * (1.0 / ratio).max(1.0)),
    )
}
