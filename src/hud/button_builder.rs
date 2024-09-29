use bevy::prelude::{
    AlignItems, BuildChildren, ButtonBundle, ChildBuilder, Component, JustifyContent, Style,
    TextBundle, TextStyle, Val,
};
use std::fmt;

pub(crate) struct ButtonBuilder<C: Component, D: fmt::Display> {
    caption: D,
    text_style: TextStyle,
    style: Style,
    context: C,
}

impl<C: Component, D: fmt::Display> ButtonBuilder<C, D> {
    /// 70px wide, dynamic height
    pub(crate) fn new(caption: D, text_style: TextStyle, context: C) -> Self {
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
            context,
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

    pub(crate) fn spawn(self, parent: &mut ChildBuilder) {
        parent
            .spawn(ButtonBundle {
                style: self.style,
                ..ButtonBundle::default()
            })
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    format!("{}", self.caption),
                    self.text_style,
                ));
            })
            .insert(self.context);
    }
}
