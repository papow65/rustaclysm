use crate::GameplayStart;
use bevy::ecs::{change_detection::Tick, system::SystemParam};
use bevy::prelude::{Local, Res};

/// This behaves like [`Local`], but resets at the end of every gameplay.
#[derive(SystemParam)]
pub struct GameplayLocal<'w, 's, T: Default + Send + 'static> {
    current: Res<'w, GameplayStart>,
    last: Local<'s, Tick>,
    wrapped: Local<'s, T>,
}

impl<T: Default + Send + 'static> GameplayLocal<'_, '_, T> {
    pub fn get(&mut self) -> &mut T {
        if *self.last != **self.current {
            *self.last = **self.current;
            *self.wrapped = Default::default();
        }
        &mut self.wrapped
    }
}
