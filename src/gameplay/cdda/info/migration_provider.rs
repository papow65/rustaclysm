use crate::gameplay::cdda::info::info_map::InfoMap;
use cdda_json_files::{
    CommonItemInfo, InfoId, ItemMigration, LinkProvider, UntypedInfoId, VehiclePartInfo,
    VehiclePartMigration,
};
use std::sync::{Arc, Mutex};

pub(super) struct ItemMigrationProvider<'a> {
    pub(super) info_map: &'a InfoMap<CommonItemInfo>,
    pub(super) migrations: &'a InfoMap<ItemMigration>,
    pub(super) variant: &'a Mutex<Option<Arc<str>>>,
}

impl LinkProvider<CommonItemInfo> for ItemMigrationProvider<'_> {
    fn get_option(&self, info_id: &InfoId<CommonItemInfo>) -> Option<&Arc<CommonItemInfo>> {
        self.info_map.get(info_id).ok().or_else(|| {
            let mut variant = self.variant.lock().expect("Variant should not be poisoned");
            let migration_id = if variant.is_some() {
                UntypedInfoId::new_suffix(&info_id.fallback_name(), variant.as_deref()).into()
            } else {
                info_id.untyped().clone().into()
            };
            self.migrations
                .get(&migration_id)
                .ok()
                .and_then(|migration| {
                    self.info_map
                        .get(&migration.replace)
                        .ok()
                        .inspect(|_| variant.clone_from(&migration.variant))
                })
        })
    }
}

pub(super) struct VehiclePartMigrationProvider<'a> {
    pub(super) info_map: &'a InfoMap<VehiclePartInfo>,
    pub(super) migrations: &'a InfoMap<VehiclePartMigration>,
}

impl LinkProvider<VehiclePartInfo> for VehiclePartMigrationProvider<'_> {
    fn get_option(&self, info_id: &InfoId<VehiclePartInfo>) -> Option<&Arc<VehiclePartInfo>> {
        self.info_map.get(info_id).ok().or_else(|| {
            self.migrations
                .get(&info_id.untyped().clone().into())
                .ok()
                .and_then(|migration| self.info_map.get(&migration.to).ok())
        })
    }
}
