use crate::gameplay::{
    Explored, GameplayScreenState, MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset,
    SubzoneLevelEntities,
};
use crate::hud::{Fonts, DEFAULT_BUTTON_COLOR, HARD_TEXT_COLOR};
use crate::loading::LoadingState;
use bevy::prelude::{
    AlignItems, Assets, BuildChildren, ChildBuild, Commands, GlobalZIndex, JustifyContent, Local,
    NextState, Node, PositionType, Res, ResMut, StateScoped, Text, Val,
};

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn spawn_loading(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Node::default()
            },
            GlobalZIndex(3),
            StateScoped(LoadingState),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Node::default()
                    },
                    DEFAULT_BUTTON_COLOR,
                ))
                .with_children(|parent| {
                    parent.spawn((Text::from("Loading..."), HARD_TEXT_COLOR, fonts.large()));
                });
        });
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn finish_loading(
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    overmap_assets: Res<Assets<OvermapAsset>>,
    overmap_buffer_assets: Res<Assets<OvermapBufferAsset>>,
    map_assets: Res<Assets<MapAsset>>,
    map_memory_assets: Res<Assets<MapMemoryAsset>>,
    explored: Res<Explored>,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    mut counter: Local<u8>,
) {
    if 3 < *counter {
        println!(
            "Loading status {}: o {}, ob {}, m {}, mm {}, e {:?}, and sle {:?}",
            *counter,
            overmap_assets.len(),
            overmap_buffer_assets.len(),
            map_assets.len(),
            map_memory_assets.len(),
            explored.loaded(),
            subzone_level_entities.loaded(),
        );

        // subzone_level_entities sometimes fails to load for unknown reason. In that case, we give control back to the user after a delay.
        let subzones_loaded = subzone_level_entities.loaded() || *counter == u8::MAX;

        if subzones_loaded
            && 0 < overmap_assets.len()
            && 0 < overmap_buffer_assets.len()
            && 0 < map_assets.len()
            && explored.loaded()
        {
            eprintln!("Loading complete");
            next_gameplay_state.set(GameplayScreenState::Base);
        }
    }

    *counter = counter.saturating_add(1);
}
