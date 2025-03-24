use bevy::prelude::{Node, PositionType, UiRect, Val};

#[must_use]
pub fn panel_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        padding: UiRect::all(Val::Px(5.0)),
        ..Node::DEFAULT
    }
}
