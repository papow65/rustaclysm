use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::{Component, Node, Val},
};

#[derive(Component, Default)]
pub(crate) struct ScrollingList {
    position: f32,
}

impl ScrollingList {
    /** Returns the new distance from the top */
    #[must_use]
    pub(crate) fn scroll(
        &mut self,
        my_node: &Node,
        parent_node: &Node,
        mouse_wheel_event: &MouseWheel,
    ) -> Val {
        let dy = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
            MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };
        self.adjust(my_node, parent_node, dy)
    }

    #[must_use]
    pub(crate) fn resize(&mut self, my_node: &Node, parent_node: &Node) -> Val {
        self.adjust(my_node, parent_node, 0.0)
    }

    #[must_use]
    pub(crate) fn adjust(&mut self, my_node: &Node, parent_node: &Node, dy: f32) -> Val {
        let items_height = my_node.size().y;
        let parent_height = parent_node.size().y;
        let max_scroll = (items_height - parent_height).max(0.);
        self.position += dy;
        self.position = self.position.clamp(-max_scroll, 0.);
        Val::Px(self.position)
    }
}
