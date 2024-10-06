use crate::hud::SOFT_TEXT_COLOR;
use crate::keyboard::{Key, KeyBinding};
use bevy::ecs::system::SystemId;
use bevy::prelude::{
    AlignItems, BuildChildren, Bundle, ButtonBundle, ChildBuild, ChildBuilder, Commands, Component,
    Entity, In, JustifyContent, NodeBundle, PositionType, Style, SystemInput, TextBundle,
    TextStyle, Val,
};
use std::fmt;

#[derive(Debug, Component)]
pub(crate) struct RunButton<I: SystemInput>
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
        commands.run_system_with_input(self.system, self.context.clone());
    }
}

pub(crate) struct ButtonBuilder<D: fmt::Display, I: SystemInput> {
    caption: D,
    text_style: TextStyle,
    style: Style,
    system: SystemId<I, ()>,
    key_binding: Option<(Key, KeyBinding<(), ()>)>,
}

impl<D: fmt::Display, I: SystemInput> ButtonBuilder<D, I>
where
    <I as SystemInput>::Inner<'static>: fmt::Debug,
    (ButtonBundle, RunButton<I>): Bundle,
{
    /// 70px wide, dynamic height
    pub(crate) fn new(caption: D, text_style: TextStyle, system: SystemId<I, ()>) -> Self {
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
        key_binding_system: SystemId<In<Entity>, ()>,
    ) -> Self {
        self.key_binding = key
            .map(Into::into)
            .map(|key| (key, KeyBinding::new(vec![key], key_binding_system.into())));
        self
    }

    pub(crate) fn spawn(
        self,
        parent: &mut ChildBuilder,
        context: <I as SystemInput>::Inner<'static>,
    ) {
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
