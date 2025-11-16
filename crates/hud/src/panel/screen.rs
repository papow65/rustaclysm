use crate::{DEFAULT_SCROLLBAR_COLOR, PANEL_COLOR, SMALL_SPACING};
use bevy::picking::hover::Hovered;
use bevy::prelude::{
    AlignItems, BorderRadius, Bundle, Children, Commands, Component, DespawnOnExit, Entity,
    FlexDirection, JustifyContent, Node, Overflow, Pickable, PositionType, Spawn,
    SpawnRelated as _, States, UiRect, Val, Visibility, WithOneRelated, children,
};
use bevy::ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};

/// Indicator component, so that other parts of the ui can adapt.
#[derive(Debug, Component)]
#[component(immutable)]
pub struct LargeNode;

const SCREEN_MARGINS: UiRect = UiRect::px(10.0, 365.0, 10.0, 10.0);
const SMALL_PADDING: UiRect = UiRect::all(SMALL_SPACING);

/// Returns the entity of the content node
#[must_use]
pub fn scroll_screen<S: States>(commands: &mut Commands, state: S) -> Entity {
    let (main_panel, content_entity) = scroll_panel_with_content_entity(commands);

    spawn_panel_root(commands, state, main_panel);

    content_entity
}

#[must_use]
pub fn scroll_panel_with_content_entity(commands: &mut Commands) -> (Spawn<impl Bundle>, Entity) {
    let content_entity = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                padding: SMALL_PADDING,
                overflow: Overflow::scroll_y(),
                ..Node::default()
            },
            Pickable::default(),
        ))
        .id();

    (scroll_panel(true, content_entity), content_entity)
}

#[must_use]
pub fn scroll_panel(limit_height: bool, content_entity: Entity) -> Spawn<impl Bundle> {
    Spawn((
        Node {
            width: Val::Percent(100.0),
            height: if limit_height {
                Val::Percent(100.0)
            } else {
                Val::Auto
            },
            overflow: Overflow::clip_y(),
            ..Node::default()
        },
        Pickable::IGNORE,
        Children::spawn((
            WithOneRelated(content_entity),
            Spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Px(6.0),
                    height: Val::Percent(100.0),
                    ..Node::default()
                },
                Scrollbar {
                    orientation: ControlOrientation::Vertical,
                    target: content_entity,
                    min_thumb_length: 12.0,
                },
                Visibility::Hidden,
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        ..Node::default()
                    },
                    Hovered::default(),
                    DEFAULT_SCROLLBAR_COLOR,
                    BorderRadius::all(Val::Px(3.0)),
                    CoreScrollbarThumb,
                )],
            )),
        )),
    ))
}

pub fn spawn_panel_root(
    commands: &mut Commands,
    state: impl States,
    content_panel: Spawn<impl Bundle>,
) {
    commands.spawn((
        // Entire screen
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Node::default()
        },
        DespawnOnExit(state),
        Pickable::IGNORE,
        children![(
            // Main panel
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto, // 100% minus the margin heights
                margin: SCREEN_MARGINS,
                ..Node::default()
            },
            PANEL_COLOR,
            LargeNode,
            Pickable::IGNORE,
            Children::spawn(content_panel)
        )],
    ));
}
