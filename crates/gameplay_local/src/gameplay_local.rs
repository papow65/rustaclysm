use crate::{GameplayCount, GameplayCounter};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Local, Res};

/// This behaves like [`Local`], but resets at the end of every gameplay.
#[derive(SystemParam)]
pub struct GameplayLocal<'w, 's, T: Default + Send + 'static> {
    current: Res<'w, GameplayCounter>,
    last: Local<'s, GameplayCount>,
    wrapped: Local<'s, T>,
}

impl<T: Default + Send + 'static> GameplayLocal<'_, '_, T> {
    pub fn get(&mut self) -> &mut T {
        let reset = self.current.get() != *self.last;
        *self.last = self.current.get();
        if reset {
            *self.wrapped = Default::default();
        }
        &mut self.wrapped
    }
}
