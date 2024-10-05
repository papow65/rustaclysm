use crate::hud::SOFT_TEXT_COLOR;
use crate::keyboard::{Key, KeyBinding};
use bevy::ecs::system::SystemId;
use bevy::prelude::{
    AlignItems, BuildChildren, ButtonBundle, ChildBuilder, Commands, Component, Entity,
    JustifyContent, NodeBundle, PositionType, Style, TextBundle, TextStyle, Val,
};
use std::fmt;

pub(crate) trait RunButtonContext: Clone + Send + Sync + Sized + 'static {}

impl RunButtonContext for () {}

#[derive(Debug, Component)]
pub(crate) struct RunButton<C: RunButtonContext> {
    system: SystemId<C, ()>,
    context: C,
}

impl<C: RunButtonContext> RunButton<C> {
    pub(super) fn run(&self, commands: &mut Commands) {
        commands.run_system_with_input(self.system, self.context.clone());
    }
}

pub(crate) struct ButtonBuilder<C: RunButtonContext, D: fmt::Display> {
    caption: D,
    text_style: TextStyle,
    style: Style,
    system: SystemId<C, ()>,
    key_binding: Option<(Key, KeyBinding<(), ()>)>,
}

impl<C: RunButtonContext, D: fmt::Display> ButtonBuilder<C, D> {
    /// 70px wide, dynamic height
    pub(crate) fn new(caption: D, text_style: TextStyle, system: SystemId<C, ()>) -> Self {
        Self {
            caption,
            text_style,
            style: Style {
                width: Val::Px(70.0),
                height: Val::Auto,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Style::default()
            },
            system,
            key_binding: None,
        }
    }

    /// 250px wide, 70px high
    pub(crate) const fn large(mut self) -> Self {
        self.style.width = Val::Px(250.0);
        self.style.height = Val::Px(70.0);
        self
    }

    pub(crate) fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub(crate) fn key_binding<K: Into<Key>>(
        mut self,
        key: Option<K>,
        key_binding_system: SystemId<Entity, ()>,
    ) -> Self {
        self.key_binding = key
            .map(Into::into)
            .map(|key| (key, KeyBinding::new(vec![key], key_binding_system.into())));
        self
    }

    pub(crate) fn spawn(self, parent: &mut ChildBuilder, context: C) {
        let mut entity_commands = parent.spawn((
            ButtonBundle {
                style: self.style,
                ..ButtonBundle::default()
            },
            RunButton {
                system: self.system,
                context,
            },
        ));

        entity_commands.with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                format!("{}", self.caption),
                self.text_style.clone(),
            ));
        });

        if let Some((key, key_binding)) = self.key_binding {
            entity_commands.insert(key_binding);

            entity_commands.with_children(|parent| {
                let mut key_text_style = self.text_style;
                key_text_style.color = SOFT_TEXT_COLOR;
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::End,
                            align_items: AlignItems::Center,
                            ..Style::default()
                        },
                        ..NodeBundle::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            match key {
                                Key::Character(c) => format!("[{c}] "),
                                Key::Code(c) => format!("[{c:?}] "),
                            },
                            key_text_style,
                        ));
                    });
            });
        }
    }
}
