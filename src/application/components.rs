use bevy::prelude::{Component, States};
use std::marker::PhantomData;

#[derive(Default, Component)]
pub(crate) struct StateBound<T: States> {
    _phantom: PhantomData<T>,
}
