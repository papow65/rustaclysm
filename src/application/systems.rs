use bevy::prelude::{Query, Window};

pub(super) fn maximize_window(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.set_maximized(true);
    }
}
