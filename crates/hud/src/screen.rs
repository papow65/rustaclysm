use crate::{DEFAULT_SCROLLBAR_COLOR, PANEL_COLOR, SMALL_SPACING, SelectionList};
use bevy::picking::hover::Hovered;
use bevy::prelude::{
    AlignItems, BorderRadius, Bundle, Children, Commands, Component, DespawnOnExit, Display,
    Entity, FlexDirection, JustifyContent, Node, Overflow, Pickable, PositionType,
    RepeatedGridTrack, Spawn, SpawnRelated as _, States, UiRect, Val, Visibility, WithOneRelated,
    children,
};
use bevy::ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};
use util::Maybe;

/// Indicator component, so that other parts of the ui can adapt.
#[derive(Debug, Component)]
#[component(immutable)]
pub struct LargeNode;

const SCREEN_MARGINS: UiRect = UiRect::px(10.0, 365.0, 10.0, 10.0);
const SMALL_PADDING: UiRect = UiRect::all(SMALL_SPACING);

/// Returns the entities of the list node and the detail node
pub fn selection_list_detail_screen<S: States>(
    commands: &mut Commands,
    state: S,
) -> (Entity, Entity) {
    let (list_panel, list_entity) = scroll_panel(commands, Some(SelectionList::default()));
    let (detail_panel, detail_entity) = scroll_panel(commands, None);

    let content_panel = Spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            ..Node::default()
        },
        Pickable::IGNORE,
        Children::spawn((list_panel, detail_panel)),
    ));

    spawn_root(commands, state, content_panel);

    (list_entity, detail_entity)
}

/// Returns the entity of the content node
pub fn scroll_screen<S: States>(commands: &mut Commands, state: S) -> Entity {
    let (main_panel, content_entity) = scroll_panel(commands, None);

    spawn_root(commands, state, main_panel);

    content_entity
}

fn scroll_panel(
    commands: &mut Commands,
    selection_list: Option<SelectionList>,
) -> (Spawn<impl Bundle>, Entity) {
    let content_node = commands
        .spawn((
            Node {
                //width: Val::Px(300.0),  //Val::Auto, TODO
                //height: Val::Px(300.0), //Val::Auto,TODO
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                padding: SMALL_PADDING,
                overflow: Overflow::scroll_y(),
                ..Node::default()
            },
            Pickable::default(),
            Maybe(selection_list),
        ))
        .id();

    let main_panel = Spawn((
        Node {
            width: Val::Percent(100.0),  //TODO
            height: Val::Percent(100.0), //TODO
            display: Display::Grid,
            grid_template_columns: vec![
                RepeatedGridTrack::flex(1, 1.0),
                RepeatedGridTrack::auto(1),
            ],
            grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.0)],
            column_gap: SMALL_SPACING,
            overflow: Overflow::clip_y(),
            ..Node::default()
        },
        Pickable::IGNORE,
        Children::spawn((
            WithOneRelated(content_node),
            Spawn((
                Node {
                    width: Val::Px(8.0),
                    ..Node::default()
                },
                Scrollbar {
                    orientation: ControlOrientation::Vertical,
                    target: content_node,
                    min_thumb_length: 8.0,
                },
                Visibility::Hidden,
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        ..Node::default()
                    },
                    Hovered::default(),
                    DEFAULT_SCROLLBAR_COLOR,
                    BorderRadius::all(Val::Px(4.0)),
                    CoreScrollbarThumb,
                )],
            )),
        )),
    ));

    (main_panel, content_node)
}

fn spawn_root(commands: &mut Commands, state: impl States, content_panel: Spawn<impl Bundle>) {
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
