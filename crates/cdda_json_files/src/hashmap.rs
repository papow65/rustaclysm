use ahash::AHasher;
use std::hash::BuildHasherDefault;

// Copied from Bevy
pub type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<AHasher>>;
