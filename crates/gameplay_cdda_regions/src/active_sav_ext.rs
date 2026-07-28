use crate::{SavPath, WorldPath};
use gameplay_cdda_active_sav::ActiveSav;

pub(crate) trait ActiveSavExt {
    #[must_use]
    fn sav_path(&self) -> SavPath;

    #[must_use]
    fn world_path(&self) -> WorldPath;
}

impl ActiveSavExt for ActiveSav {
    fn sav_path(&self) -> SavPath {
        SavPath::init(self.path().clone())
    }

    fn world_path(&self) -> WorldPath {
        WorldPath::init(
            self.path()
                .parent()
                .expect("Path to sav file should have a parent directory")
                .to_path_buf(),
        )
    }
}
