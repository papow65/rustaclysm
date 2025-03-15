use crate::gameplay::{TypeId, cdda::Error};
use bevy::prelude::{debug, error, warn};
use bevy::utils::HashMap;
use cdda_json_files::{
    Bash, BashItem, BashItems, CommonItemInfo, FurnitureInfo, InfoId, ItemMigration,
    ItemWithCommonInfo, Quality, Recipe, Requirement, TerrainInfo, VehiclePartInfo,
    VehiclePartMigration,
};
use serde::de::DeserializeOwned;
use std::{any::type_name, sync::Arc};

pub(crate) struct InfoMap<T> {
    pub(crate) map: HashMap<InfoId, Arc<T>>,
}

impl<T: DeserializeOwned + 'static> InfoMap<T> {
    pub(crate) fn new(
        all: &mut HashMap<TypeId, HashMap<InfoId, serde_json::Map<String, serde_json::Value>>>,
        type_ids: &[TypeId],
    ) -> Self {
        let mut map = HashMap::default();
        for type_id in type_ids {
            let objects = all
                .remove(type_id)
                .unwrap_or_else(|| panic!("Type {type_id:?} not found"));
            for (id, object_properties) in objects {
                //trace!("{:#?}", &object_properties);
                match serde_json::from_value::<T>(serde_json::Value::Object(object_properties)) {
                    Ok(info) => {
                        map.insert(id, Arc::new(info));
                    }
                    Err(error) => {
                        error!(
                            "Failed loading json for {:?} {:?}: {error:#?}",
                            type_name::<T>(),
                            &id
                        );
                    }
                }
            }
        }

        Self { map }
    }

    pub(crate) fn get(&self, id: &InfoId) -> Result<&Arc<T>, Error> {
        self.map.get(id).ok_or_else(|| Error::UnknownObject {
            _id: id.clone(),
            _type: type_name::<T>(),
        })
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &Arc<T>> {
        self.map.values()
    }
}

impl InfoMap<CommonItemInfo> {
    pub(crate) fn link_common_items(&self, qualities: &InfoMap<Quality>) {
        for common_item_info in self.map.values() {
            common_item_info
                .qualities
                .finalize(&qualities.map, "quality");
        }
    }
}

impl InfoMap<FurnitureInfo> {
    pub(crate) fn link_furniture(&self, common_item_infos: &InfoMap<CommonItemInfo>) {
        for furniture in self.map.values() {
            furniture
                .crafting_pseudo_item
                .finalize(&common_item_infos.map, "pseudo item");
            if let Some(bash) = &furniture.bash {
                // Not expected to occur
                let terrain = InfoMap {
                    map: HashMap::default(),
                };

                link_bash(bash, &terrain, self, common_item_infos, "furniture");
            }
        }
    }
}

impl InfoMap<Recipe> {
    pub(crate) fn link_recipes(
        &self,
        qualities: &InfoMap<Quality>,
        common_item_infos: &InfoMap<CommonItemInfo>,
    ) {
        for recipe in self.map.values() {
            for required_quality in &recipe.qualities.0 {
                required_quality
                    .quality
                    .finalize(&qualities.map, "required quality for recipe");
            }

            recipe.result.finalize(&common_item_infos.map, "recipe");
        }
    }
}

impl InfoMap<Requirement> {
    pub(crate) fn link_requirements(&self, qualities: &InfoMap<Quality>) {
        for requirement in self.map.values() {
            for required_quality in &requirement.qualities.0 {
                required_quality
                    .quality
                    .finalize(&qualities.map, "required quality for requirement");
            }
        }
    }
}

impl InfoMap<TerrainInfo> {
    pub(crate) fn fix_and_link_terrain(
        &mut self,
        furniture: &InfoMap<FurnitureInfo>,
        common_item_infos: &InfoMap<CommonItemInfo>,
    ) {
        if self.map.remove(&InfoId::new("t_null")).is_some() {
            warn!("The terrain t_null was not expected to be present");
        }

        for terrain in self.map.values() {
            terrain.open.finalize(&self.map, "open terrain");
            terrain.close.finalize(&self.map, "closed terrain");
            if let Some(bash) = &terrain.bash {
                link_bash(bash, self, furniture, common_item_infos, "terrain");
            }
        }
    }
}

fn link_bash(
    bash: &Bash,
    terrain_info: &InfoMap<TerrainInfo>,
    furniture_info: &InfoMap<FurnitureInfo>,
    common_item_infos: &InfoMap<CommonItemInfo>,
    name: &str,
) {
    bash.terrain.finalize(
        &terrain_info.map,
        format!("terrain for bashed {name}").as_str(),
    );
    bash.furniture.finalize(
        &furniture_info.map,
        format!("terrain for bashed {name}").as_str(),
    );
    if let Some(BashItems::Explicit(bash_items)) = &bash.items {
        for bash_item in bash_items {
            if let BashItem::Single(item_occurrence) = bash_item {
                item_occurrence.item.finalize(
                    &common_item_infos.map,
                    format!("terrain for bashed {name}").as_str(),
                );
            }
        }
    }
}

impl InfoMap<VehiclePartInfo> {
    pub(crate) fn add_vehicle_part_migrations(
        &mut self,
        vehicle_part_migrations: &HashMap<InfoId, Arc<VehiclePartMigration>>,
    ) {
        // TODO Make this recursive
        for (migration_from, migration) in vehicle_part_migrations {
            if let Ok(new) = self.get(migration_from).cloned() {
                self.map.insert(migration.from.clone(), new);
            }
        }
    }
}

pub(crate) struct ItemInfoMapLoader<'a> {
    pub(crate) enriched_json_infos:
        &'a mut HashMap<TypeId, HashMap<InfoId, serde_json::Map<String, serde_json::Value>>>,
    pub(crate) item_migrations: HashMap<InfoId, Arc<ItemMigration>>,
    pub(crate) common_item_infos: &'a mut InfoMap<CommonItemInfo>,
}

impl ItemInfoMapLoader<'_> {
    pub(crate) fn item_extract<T>(&mut self, type_ids: &[TypeId]) -> InfoMap<T>
    where
        T: DeserializeOwned + ItemWithCommonInfo + 'static,
    {
        let mut items = InfoMap::<T>::new(self.enriched_json_infos, type_ids);

        // TODO Make this recursive
        for (migration_from, migration) in &self.item_migrations {
            if let Ok(new) = items.get(migration_from).cloned() {
                items.map.insert(migration.replace.clone(), new);
            }
        }

        for (id, item_info) in &mut items.map {
            self.common_item_infos
                .map
                .insert(id.clone(), item_info.common());
        }

        debug!(
            "{}x {}, and {}x common items",
            items.map.len(),
            std::any::type_name::<T>(),
            self.common_item_infos.map.len()
        );

        items
    }
}
