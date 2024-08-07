use crate::background::{component::Background, state::BackgroundState};
use crate::common::Paths;
use bevy::prelude::{
    AssetServer, Camera, Commands, ImageBundle, PositionType, Query, Res, StateScoped, Style,
    UiImage, Val, With, ZIndex,
};

const BACKGROUND_WIDTH: f32 = 1522.0;
const BACKGROUND_HEIGHT: f32 = 1009.0;
const BACKGROUND_NAME: &str = "on_the_run.png";

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<&Camera>,
) {
    let background_scale = background_scale(cameras.get_single().ok());
    let background_image = asset_server.load(Paths::backgrounds_path().join(BACKGROUND_NAME));
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
            z_index: ZIndex::Global(2),
            ..ImageBundle::default()
        },
        Background,
        StateScoped(BackgroundState),
    ));
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn resize_background(
    cameras: Query<&Camera>,
    mut backgrounds: Query<&mut Style, With<Background>>,
) {
    if let Ok(mut style) = backgrounds.get_single_mut() {
        (style.width, style.height) = background_scale(cameras.get_single().ok());
    }
}

fn background_scale(camera: Option<&Camera>) -> (Val, Val) {
    let ratio = if let Some(camera_size) = camera.and_then(Camera::physical_target_size) {
        camera_size.y as f32 * BACKGROUND_WIDTH / (BACKGROUND_HEIGHT * camera_size.x as f32)
    } else {
        eprintln!("No camera size available (yet?) to resize the background to");
        1.0
    };

    (
        Val::Percent(100.0 * ratio.max(1.0)),
        Val::Percent(100.0 * (1.0 / ratio).max(1.0)),
    )
}
