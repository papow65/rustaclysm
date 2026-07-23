use crate::{Fonts, text::DebugTextShown};
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::prelude::{Command, Component, Entity, World};

/// Also adds the appropriate font, using a hook
#[derive(Clone, Copy, Debug, Component)]
#[component(immutable)]
#[component(on_add=on_add_debug)]
pub struct DebugText;

/// A hook that runs whenever [`DebugText`] is added to an entity.
///
/// Generates a [`DebugTextCommand`].
pub(crate) fn on_add_debug(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    // Component hooks can't perform structural changes, so we need to rely on commands.
    world.commands().queue(DebugTextCommand { entity });
}

struct DebugTextCommand {
    entity: Entity,
}

impl Command for DebugTextCommand {
    type Out = ();

    fn apply(self, world: &mut World) {
        let debug_text_shown = world.resource::<DebugTextShown>();
        let font = debug_text_shown.text_font(Fonts::regular());

        world
            .get_entity_mut(self.entity)
            .expect("DebugText component should have been found")
            .insert(font);
    }
}
