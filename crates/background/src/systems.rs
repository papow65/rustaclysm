use crate::{component::Background, handle::BackgroundHandle, state::BackgroundState};
use bevy::prelude::{
    AssetServer, Camera2d, Commands, DespawnOnExit, GlobalZIndex, ImageNode, Node, PositionType,
    Res, Single, Val, Window, With, debug, warn,
};
use util::AssetPaths;

const BACKGROUND_WIDTH: f32 = 1522.0;
const BACKGROUND_HEIGHT: f32 = 1009.0;
const BACKGROUND_NAME: &str = "on_the_run.png";

#[expect(clippy::needless_pass_by_value)]
pub(super) fn load_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load(AssetPaths::backgrounds().join(BACKGROUND_NAME));
    commands.insert_resource(BackgroundHandle(handle));
    debug!("Background handle created");
}

pub(super) fn spawn_background_camera(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(BackgroundState)));
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_background(
    mut commands: Commands,
    background_handle: Res<BackgroundHandle>,
    window: Option<Single<&Window>>,
) {
    let background_scale = background_scale(window.map(|w| *w));
    commands.spawn((
        ImageNode {
            image: background_handle.0.clone(),
            ..ImageNode::default()
        },
        Node {
            position_type: PositionType::Absolute,
            width: background_scale.0,
            height: background_scale.1,
            ..Node::default()
        },
        GlobalZIndex(1),
        Background,
        DespawnOnExit(BackgroundState),
    ));
}

pub(super) fn resize_background(
    window: Option<Single<&Window>>,
    background: Option<Single<&mut Node, With<Background>>>,
) {
    if let Some(mut background) = background {
        let &mut ref mut style = &mut *background;
        (style.width, style.height) = background_scale(window.map(|w| *w));
    }
}

fn background_scale(window: Option<&Window>) -> (Val, Val) {
    let ratio = if let Some(window) = window {
        window.resolution.height() * BACKGROUND_WIDTH
            / (BACKGROUND_HEIGHT * window.resolution.width())
    } else {
        warn!("No window size available (yet?) to resize the background to");
        1.0
    };

    (
        Val::Percent(100.0 * ratio.max(1.0)),
        Val::Percent(100.0 * (1.0 / ratio).max(1.0)),
    )
}
