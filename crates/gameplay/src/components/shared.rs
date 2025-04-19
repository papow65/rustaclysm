use bevy::prelude::Component;
use std::{ops::Deref, sync::Arc};

#[derive(Debug, Component)]
#[component(immutable)]
pub(crate) struct Shared<T>(Arc<T>);

impl<T> Shared<T> {
    pub(crate) const fn new(arc: Arc<T>) -> Self {
        Self(arc)
    }
}

impl<T> AsRef<Arc<T>> for Shared<T> {
    fn as_ref(&self) -> &Arc<T> {
        &self.0
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}
