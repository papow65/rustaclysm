//! Gameplay object components shared among multiple types of gameplay objects:
//! items, corpses, actors, furniture, etc.

mod closeable;
mod corpse;
mod damage;
mod healing;
mod hurdle;
mod life;
mod mobile;
mod object_name;
mod obstacle;
mod opaque;
mod openable;
mod standard_integrity;

pub use closeable::Closeable;
pub use corpse::{Corpse, CorpseRaise};
pub use damage::Damage;
pub use healing::Healing;
pub use hurdle::Hurdle;
pub use life::Life;
pub use mobile::Mobile;
pub use object_name::ObjectName;
pub use obstacle::Obstacle;
pub use opaque::Opaque;
pub use openable::Openable;
pub use standard_integrity::StandardIntegrity;
