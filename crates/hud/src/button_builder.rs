use crate::SOFT_TEXT_COLOR;
use bevy::ecs::system::SystemId;
use bevy::prelude::{
    AlignItems, Bundle, Button, ChildSpawnerCommands, Commands, Component, Entity, In,
    JustifyContent, Node, PositionType, SystemInput, Text, TextColor, TextFont, Val,
};
use keyboard::{Key, KeyBinding};
use std::fmt;

#[derive(Debug, Component)]
#[component(immutable)]
pub struct RunButton<I: SystemInput>
where
    <I as SystemInput>::Inner<'static>: fmt::Debug,
{
    system: SystemId<I, ()>,
    context: <I as SystemInput>::Inner<'static>,
}

impl<I: SystemInput + 'static> RunButton<I>
where
    <I as SystemInput>::Inner<'static>: Clone + fmt::Debug + Send + 'static,
{
    pub(super) fn run(&self, commands: &mut Commands) {
        commands.run_system_with(self.system, self.context.clone());
    }
}

pub struct ButtonBuilder<D: fmt::Display, I: SystemInput> {
    caption: D,
    text_color: TextColor,
    text_font: TextFont,
    node: Node,
    system: SystemId<I, ()>,
    key_binding: Option<(Key, KeyBinding<(), ()>)>,
}

impl<D: fmt::Display, I: SystemInput> ButtonBuilder<D, I>
where
    <I as SystemInput>::Inner<'static>: fmt::Debug,
    (Button, Node, RunButton<I>): Bundle,
{
    /// 70px wide, dynamic height
    pub fn new(
        caption: D,
        text_color: TextColor,
        text_font: TextFont,
        system: SystemId<I, ()>,
    ) -> Self {
        Self {
            caption,
            text_color,
            text_font,
            node: Node {
                width: Val::Px(70.0),
                height: Val::Auto,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Node::default()
            },
            system,
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

    pub fn spawn(
        self,
        parent: &mut ChildSpawnerCommands,
        context: <I as SystemInput>::Inner<'static>,
    ) {
        let mut entity_commands = parent.spawn((
            Button,
            self.node,
            RunButton {
                system: self.system,
                context,
            },
        ));

        entity_commands.with_children(|parent| {
            parent.spawn((
                Text(format!("{}", self.caption)),
                self.text_color,
                self.text_font.clone(),
            ));
        });

        if let Some((key, key_binding)) = self.key_binding {
            entity_commands.insert(key_binding);

            entity_commands.with_children(|parent| {
                parent
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Center,
                        ..Node::default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            Text(match key {
                                Key::Character(c) => format!("[{c}] "),
                                Key::Code(c) => format!("[{c:?}] "),
                            }),
                            SOFT_TEXT_COLOR,
                            self.text_font,
                        ));
                    });
            });
        }
    }
}
