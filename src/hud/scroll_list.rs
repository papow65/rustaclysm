use bevy::ecs::component::{ComponentHooks, StorageType};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::{Component, Interaction, JustifyContent, Node, Style, Transform, Val};

// Manually deriving `Component`
#[derive(Debug, Default)]
pub(crate) struct ScrollList {
    /// Smaller than or equal to 0.0, this matches with `Style.top`.
    position: f32,
}

impl ScrollList {
    /// Returns the new distance from the top
    #[must_use]
    pub(crate) fn scroll(
        &mut self,
        my_node: &Node,
        parent_node: &Node,
        parent_style: &Style,
        mouse_wheel_event: &MouseWheel,
    ) -> Val {
        self.position += match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * 20.0,
            MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };
        self.adjust(my_node, parent_node, parent_style)
    }

    /// Returns the new distance from the top
    #[must_use]
    pub(crate) fn follow(
        &mut self,
        child_transform: &Transform,
        child_node: &Node,
        my_node: &Node,
        parent_node: &Node,
        parent_style: &Style,
    ) -> Val {
        let child_top =
            my_node.size().y / 2.0 + child_transform.translation.y - child_node.size().y / 2.0;

        //let first_viewed_top = self.position;
        //let last_viewed_top = self.position + parent_node.size().y - child_node.size().y;
        //println!("{first_viewed_top:?} <= {child_top:?} <= {last_viewed_top:?}");

        self.position = self.position.clamp(
            -child_top,
            (parent_node.size().y - child_node.size().y) - child_top,
        );

        //let first_viewed_top = self.position;
        //let last_viewed_top = self.position + parent_node.size().y - child_node.size().y;
        //println!("-> {first_viewed_top:?} <= {child_top:?} <= {last_viewed_top:?}");

        self.adjust(my_node, parent_node, parent_style)
    }

    /// Returns the new distance from the top
    #[must_use]
    pub(crate) fn resize(
        &mut self,
        my_node: &Node,
        parent_node: &Node,
        parent_style: &Style,
    ) -> Val {
        self.adjust(my_node, parent_node, parent_style)
    }

    #[must_use]
    fn adjust(&mut self, my_node: &Node, parent_node: &Node, parent_style: &Style) -> Val {
        let padding_top = Self::to_px(parent_style.padding.top);
        let padding_bottom = Self::to_px(parent_style.padding.bottom);

        let items_height = my_node.size().y + padding_top + padding_bottom;
        let parent_height = parent_node.size().y;
        let max_scroll = (items_height - parent_height).max(0.0);

        self.position = match parent_style.justify_content {
            JustifyContent::Default | JustifyContent::Start => {
                self.position.clamp(-max_scroll, 0.0)
            }
            JustifyContent::End => self.position.clamp(0.0, max_scroll),
            missing => todo!("{missing:?}"),
        };
        //println!("=> {:?}", self.position);
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

impl Component for ScrollList {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            world.commands().entity(entity).insert(Interaction::None);
        });
    }
}
