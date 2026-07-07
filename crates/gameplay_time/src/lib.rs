//! Clock, timeouts, and plugin management for game time.

mod clock;
mod plugin;
mod timeouts;

pub use self::clock::Clock;
pub use self::plugin::TimePlugin;
pub use self::timeouts::Timeouts;
