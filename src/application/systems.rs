use bevy::prelude::{Query, Window};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn maximize_window(mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        window.set_maximized(true);
    }
}
