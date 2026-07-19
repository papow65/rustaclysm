//! Stats for characters

mod aquatic;
mod health;
mod melee;
mod speed;
mod stamina;

pub use self::aquatic::Aquatic;
pub use self::health::{HealingDuration, Health};
pub use self::melee::Melee;
pub use self::speed::{BaseSpeed, WalkingMode};
pub use self::stamina::{Stamina, StaminaCost, StaminaImpact};
