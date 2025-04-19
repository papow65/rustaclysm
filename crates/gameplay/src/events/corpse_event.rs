use bevy::prelude::{Entity, Event};

pub(crate) trait CorpseChange: Clone + Send + Sync + 'static {}

#[must_use]
#[derive(Clone, Debug, Event)]
pub(crate) struct CorpseEvent<C: CorpseChange> {
    pub(crate) corpse_entity: Entity,
    pub(crate) change: C,
}

impl<C: CorpseChange> CorpseEvent<C> {
    pub(crate) const fn new(corpse_entity: Entity, change: C) -> Self {
        Self {
            corpse_entity,
            change,
        }
    }
}
