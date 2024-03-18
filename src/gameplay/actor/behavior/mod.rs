mod plugin;
mod schedule;
mod system_configs;
mod systems;

pub(super) use self::plugin::BehaviorPlugin;
pub(crate) use self::system_configs::loop_behavior_and_refresh;
