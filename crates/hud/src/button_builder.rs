use crate::SOFT_TEXT_COLOR;
use bevy::ecs::{spawn::SpawnWith, system::SystemId};
use bevy::prelude::{
    AlignItems, Bundle, Button, ChildSpawner, Children, Commands, Component, Entity, In,
    JustifyContent, Node, PositionType, Spawn, SpawnRelated as _, SystemInput, Text, TextColor,
    TextFont, Val, children,
};
use keyboard::{Key, KeyBinding};
use std::fmt;

#[derive(Debug, Component)]
#[component(immutable)]
pub struct RunButton<I: fmt::Debug + SystemInput>
where
    <I as SystemInput>::Inner<'static>: Clone + fmt::Debug,
{
    system: SystemId<I, ()>,
    context: <I as SystemInput>::Inner<'static>,
}

impl<I: fmt::Debug + SystemInput + 'static> RunButton<I>
where
    <I as SystemInput>::Inner<'static>: Clone + fmt::Debug + Send + 'static,
{
    pub(super) fn run(&self, commands: &mut Commands) {
        commands.run_system_with(self.system, self.context.clone());
    }
}

pub struct ButtonBuilder<I: fmt::Debug + SystemInput<Inner<'static>: Clone + fmt::Debug>> {
    text: Text,
    text_color: TextColor,
    text_font: TextFont,
    node: Node,
    run_button: RunButton<I>,
    key_binding: Option<(Key, KeyBinding<(), ()>)>,
}

impl<I: fmt::Debug + SystemInput + 'static> ButtonBuilder<I>
where
    <I as SystemInput>::Inner<'static>: Clone + fmt::Debug + Send + Sync,
{
    /// 70px wide, dynamic height
    pub fn new<S: Into<String>>(
        caption: S,
        text_color: TextColor,
        text_font: TextFont,
        system: SystemId<I, ()>,
        context: <I as SystemInput>::Inner<'static>,
    ) -> Self {
        Self {
            text: Text(caption.into()),
            text_color,
            text_font,
            node: Node {
                width: Val::Px(70.0),
                height: Val::Auto,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Node::default()
            },
            run_button: RunButton { system, context },
            key_binding: None,
        }
    }

    /// 250px wide, 70px high
    #[must_use]
    pub const fn large(mut self) -> Self {
        self.node.width = Val::Px(250.0);
        self.node.height = Val::Px(70.0);
        self
    }

    #[must_use]
    pub fn with_node(mut self, node: Node) -> Self {
        self.node = node;
        self
    }

    #[must_use]
    pub fn key_binding<K: Into<Key>>(
        mut self,
        key: Option<K>,
        key_binding_system: SystemId<In<Entity>, ()>,
    ) -> Self {
        self.key_binding = key
            .map(K::into)
            .map(|key| (key, KeyBinding::new(key, key_binding_system.into())));
        self
    }

    pub fn bundle(self) -> impl Bundle {
        (
            Button,
            self.node,
            // If we didn't need 'self.run_button' below, we'd use it here.
            RunButton {
                context: self.run_button.context.clone(),
                system: self.run_button.system,
            },
            Children::spawn((
                Spawn((self.text, self.text_color, self.text_font.clone())),
                SpawnWith(|parent: &mut ChildSpawner| {
                    if let Some((key, key_binding)) = self.key_binding {
                        // Conditionally adding key_binding to the parent bundle is not possible because Bundle is not dyn compatible. So we add it here.
                        parent.spawn((key_binding, self.run_button));

                        parent.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::End,
                                align_items: AlignItems::Center,
                                ..Node::default()
                            },
                            children![(
                                Text(match key {
                                    Key::Character(c) => format!("[{c}] "),
                                    Key::Code(c) => format!("[{c:?}] "),
                                }),
                                SOFT_TEXT_COLOR,
                                self.text_font,
                            )],
                        ));
                    }
                }),
            )),
        )
    }
}
