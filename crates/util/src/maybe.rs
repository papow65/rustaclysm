//! Copied from <https://github.com/Leafwing-Studios/i-cant-believe-its-not-bsn/blob/main/src/maybe.rs>
//! Licenses:
//! - <https://github.com/Leafwing-Studios/i-cant-believe-its-not-bsn?tab=Apache-2.0-1-ov-file>
//! - <https://github.com/Leafwing-Studios/i-cant-believe-its-not-bsn?tab=MIT-2-ov-file>

use bevy::ecs::component::{ComponentHooks, HookContext, Immutable, StorageType};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::{Bundle, Command, Component, Entity, World};
use core::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Maybe<B: Bundle>(pub Option<B>);

impl<B: Bundle> Component for Maybe<B> {
    type Mutability = Immutable;

    /// This is a sparse set component as it's only ever added and removed, never iterated over.
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(maybe_hook::<B>);
    }
}

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
fn maybe_hook<B: Bundle>(mut world: DeferredWorld<'_>, HookContext { entity, .. }: HookContext) {
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
        let Ok(mut entity_mut) = world.get_entity_mut(self.entity) else {
            #[cfg(debug_assertions)]
            panic!("Entity with Maybe component not found");

            #[cfg(not(debug_assertions))]
            return;
        };

        let Some(maybe_component) = entity_mut.take::<Maybe<B>>() else {
            #[cfg(debug_assertions)]
            panic!("Maybe component not found");

            #[cfg(not(debug_assertions))]
            return;
        };

        if let Some(bundle) = maybe_component.into_inner() {
            entity_mut.insert(bundle);
        }
    }
}
