//! Based on <https://github.com/Leafwing-Studios/i-cant-believe-its-not-bsn/blob/main/src/maybe.rs>
//! Licenses:
//! - <https://github.com/Leafwing-Studios/i-cant-believe-its-not-bsn?tab=Apache-2.0-1-ov-file>
//! - <https://github.com/Leafwing-Studios/i-cant-believe-its-not-bsn?tab=MIT-2-ov-file>

use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::prelude::{Bundle, Command, Component, Entity, World};
use core::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Component)]
#[component(on_add=on_add_maybe::<B>)]
pub struct Maybe<B: Bundle>(pub Option<B>);

impl<B: Bundle> Maybe<B> {
    /// Creates a new `Maybe` component of type `B` with no bundle.
    pub const NONE: Self = Self(None);

    /// Creates a new `Maybe` component with the given bundle.
    pub const fn new(bundle: B) -> Self {
        Self(Some(bundle))
    }

    /// Returns the contents of the `Maybe` component, if any.
    pub fn into_inner(self) -> Option<B> {
        self.0
    }
}

impl<B: Bundle> Default for Maybe<B> {
    /// Defaults to [`Maybe::NONE`].
    fn default() -> Self {
        Self::NONE
    }
}

/// A hook that runs whenever [`Maybe`] is added to an entity.
///
/// Generates a [`MaybeCommand`].
pub(crate) fn on_add_maybe<B: Bundle>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    // Component hooks can't perform structural changes, so we need to rely on commands.
    world.commands().queue(MaybeCommand {
        entity,
        _phantom: PhantomData::<B>,
    });
}

struct MaybeCommand<B> {
    entity: Entity,
    _phantom: PhantomData<B>,
}

impl<B: Bundle> Command for MaybeCommand<B> {
    fn apply(self, world: &mut World) {
        let mut entity_mut = world
            .get_entity_mut(self.entity)
            .expect("Maybe component should have been found");
        let maybe_component = entity_mut
            .take::<Maybe<B>>()
            .expect("Maybe component not found");
        if let Some(bundle) = maybe_component.into_inner() {
            entity_mut.insert(bundle);
        }
    }
}
