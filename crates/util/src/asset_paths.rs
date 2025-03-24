use std::path::{Path, PathBuf};

/// This represents all staticly known file paths to assets/ and its subdirectories.
pub struct AssetPaths;

impl AssetPaths {
    #[must_use]
    pub fn assets() -> &'static Path {
        Path::new("assets")
    }

    #[must_use]
    pub fn backgrounds() -> PathBuf {
        Self::assets().join("backgrounds")
    }

    #[must_use]
    pub fn data() -> PathBuf {
        Self::assets().join("data")
    }

    #[must_use]
    pub fn fonts() -> PathBuf {
        Self::assets().join("fonts")
    }

    #[must_use]
    pub fn gfx() -> PathBuf {
        Self::assets().join("gfx")
    }

    #[must_use]
    pub fn save() -> PathBuf {
        Self::assets().join("save")
    }
}
