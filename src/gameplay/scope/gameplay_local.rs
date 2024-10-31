use crate::gameplay::scope::gameplay_counter::{GameplayCount, GameplayCounter};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Local, Res};

/// This behaves like [`Local`], but resets every gameplay.
#[derive(SystemParam)]
pub(crate) struct GameplayLocal<'w, 's, T: Default + Send + 'static> {
    current: Res<'w, GameplayCounter>,
    last: Local<'s, GameplayCount>,
    wrapped: Local<'s, T>,
}

impl<'w, 's, T: Default + Send + 'static> GameplayLocal<'w, 's, T> {
    pub(crate) fn get(&mut self) -> &mut T {
        let reset = self.current.get() != *self.last;
        *self.last = self.current.get();
        if reset {
            *self.wrapped = Default::default();
        }
        &mut self.wrapped
    }
}
