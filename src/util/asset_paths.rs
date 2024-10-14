use std::path::{Path, PathBuf};

/// This represents all staticly known file paths to assets/ and its subdirectories.
///
/// See also [`ActiveSav`](`crate::gameplay::ActiveSav`)
pub(crate) struct AssetPaths;

impl AssetPaths {
    pub(crate) fn assets() -> &'static Path {
        Path::new("assets")
    }

    pub(crate) fn backgrounds() -> PathBuf {
        Self::assets().join("backgrounds")
    }

    pub(crate) fn data() -> PathBuf {
        Self::assets().join("data")
    }

    pub(crate) fn fonts() -> PathBuf {
        Self::assets().join("fonts")
    }

    pub(crate) fn gfx() -> PathBuf {
        Self::assets().join("gfx")
    }

    pub(crate) fn save() -> PathBuf {
        Self::assets().join("save")
    }
}
