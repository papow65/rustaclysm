use bevy::prelude::{ComputedNode, Vec2};

#[must_use]
pub fn max_scroll(node: &ComputedNode) -> Vec2 {
    Vec2::new(
        (node.content_size.x - node.size.x).max(0.0) * node.inverse_scale_factor,
        (node.content_size.y - node.size.y).max(0.0) * node.inverse_scale_factor,
    )
}
