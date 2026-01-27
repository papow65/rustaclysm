mod duration;
mod phrases;
mod plugin;
mod systems;

pub(super) use self::plugin::WaitingModalPlugin;

use self::duration::WaitDuration;
use self::phrases::YouWait;
use self::systems::{
    create_waiting_modal_key_bindings, create_waiting_modal_system, spawn_wait_modal,
};
