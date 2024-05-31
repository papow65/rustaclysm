use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::{Component, Node, Style, Val},
};

#[derive(Debug, Default, Component)]
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
        parent_style: &Style,
        mouse_wheel_event: &MouseWheel,
    ) -> Val {
        let dy = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
            MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };
        self.adjust(my_node, parent_node, parent_style, dy)
    }

    #[must_use]
    pub(crate) fn resize(
        &mut self,
        my_node: &Node,
        parent_node: &Node,
        parent_style: &Style,
    ) -> Val {
        self.adjust(my_node, parent_node, parent_style, 0.0)
    }

    #[must_use]
    pub(crate) fn adjust(
        &mut self,
        my_node: &Node,
        parent_node: &Node,
        parent_style: &Style,
        dy: f32,
    ) -> Val {
        let padding_top = Self::to_px(parent_style.padding.top);
        let padding_bottom = Self::to_px(parent_style.padding.bottom);

        let items_height = my_node.size().y + padding_top + padding_bottom;
        let parent_height = parent_node.size().y;
        let max_scroll = (items_height - parent_height).max(0.);
        self.position += dy;
        self.position = self.position.clamp(-max_scroll, 0.);
        Val::Px(self.position)
    }

    /// This assumes [`Val::Auto`] is used vertically
    fn to_px(val: Val) -> f32 {
        match val {
            Val::Auto => 0.0,
            Val::Px(px) => px,
            other => unimplemented!("Conversion of {:?} to pixels", other),
        }
    }
}
