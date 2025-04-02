use crate::gameplay::cdda::info::parsed_json::Enriched;
use crate::gameplay::{TypeId, cdda::Error};
use bevy::platform_support::collections::HashMap;
use bevy::prelude::{debug, error, warn};
use cdda_json_files::{
    Alternative, Bash, BashItem, BashItems, CddaItemName, CharacterInfo, CommonItemInfo, Flags,
    FurnitureInfo, InfoId, InfoIdDescription, ItemAction, ItemGroup, ItemMigration, ItemName,
    ItemWithCommonInfo, Link as _, LinkProvider, Quality, Recipe, RecipeResult,
    RequiredLinkedLater, Requirement, TerrainInfo, UntypedInfoId, VehiclePartInfo,
    VehiclePartMigration,
};
use serde::de::DeserializeOwned;
use std::{any::type_name, fmt, sync::Arc};
use units::{Mass, Volume};

pub(crate) struct InfoMap<T> {
    map: HashMap<InfoId<T>, Arc<T>>,
    aliases: HashMap<InfoId<T>, Arc<T>>,
}

impl<T: fmt::Debug + DeserializeOwned + 'static> InfoMap<T> {
    pub(super) fn new(
        all: &mut HashMap<TypeId, HashMap<UntypedInfoId, Enriched>>,
        type_id: TypeId,
    ) -> Self {
        let mut map = HashMap::default();
        let mut aliases = HashMap::default();

        let objects = all
            .remove(&type_id)
            .unwrap_or_else(|| panic!("Type {type_id:?} not found"));
        let objects_len = objects.len();
        for (id, enriched) in objects {
            //trace!("{:#?}", &object_properties);
            match serde_json::from_value::<T>(serde_json::Value::Object(enriched.fields.clone())) {
                Ok(info) => {
                    let info = Arc::new(info);
                    for alias_id in enriched.alias_ids {
                        aliases.insert(alias_id.into(), info.clone());
                    }
                    map.insert(id.into(), info);
                }
                Err(error) => {
                    error!(
                        "Failed loading json for {:?} {id:?}: {error:#?}",
                        type_name::<T>()
                    );
                    if let Some(cause) = enriched.fields.keys().find(|key| {
                        let mut copy = enriched.fields.clone();
                        copy.remove(*key);
                        serde_json::from_value::<T>(serde_json::Value::Object(copy)).is_ok()
                    }) {
                        warn!("Failure for {id:?} likely caused by the property '{cause}'");
                    }
                    debug!("Json for {id:?}: {:#?}", &enriched.fields);
                }
            }
        }

        debug!(
            "Processed {objects_len} -> {} {type_id:?} entries",
            map.len()
        );
        Self { map, aliases }
    }

    pub(crate) fn get(&self, id: &InfoId<T>) -> Result<&Arc<T>, Error> {
        self.map
            .get(id)
            .or_else(|| self.aliases.get(id))
            .ok_or_else(|| Error::UnknownObject {
                _id: InfoIdDescription::from(id.clone()),
            })
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &Arc<T>> {
        self.map.values()
    }
}

impl InfoMap<CharacterInfo> {
    pub(super) fn add_default_human(&mut self) {
        let default_human = CharacterInfo {
            id: InfoId::new("human"),
            name: ItemName::from(CddaItemName::Simple(Arc::from("Human"))),
            default_faction: Arc::from("human"),
            looks_like: Some(UntypedInfoId::new("overlay_male_mutation_SKIN_TAN")),
            volume: Some(Volume::from("80 l")),
            mass: Some(Mass::from("80 kg")),
            hp: Some(100),
            speed: 100,
            melee_dice: 2,
            melee_dice_sides: 4,
            flags: Flags::default(),
            extra: HashMap::default(),
        };
        self.map
            .insert(InfoId::new("human"), Arc::new(default_human));
    }
}

impl InfoMap<CommonItemInfo> {
    pub(super) fn link_common_items(&self, qualities: &InfoMap<Quality>) {
        for common_item_info in self.map.values() {
            for quality in &common_item_info.qualities {
                quality.id.finalize(qualities, "quality");
            }
        }
    }
}

impl InfoMap<FurnitureInfo> {
    pub(super) fn link_furniture(
        &self,
        common_item_infos: &InfoMap<CommonItemInfo>,
        item_groups: &InfoMap<ItemGroup>,
    ) {
        for furniture in self.map.values() {
            furniture
                .crafting_pseudo_item
                .finalize(common_item_infos, "pseudo item");
            if let Some(bash) = &furniture.bash {
                link_bash(
                    bash,
                    &InfoMap::default(), // No terrain expected
                    self,
                    common_item_infos,
                    item_groups,
                    "furniture",
                );
            }
        }
    }
}

impl InfoMap<Quality> {
    pub(crate) fn link_qualities(&self, item_actions: &InfoMap<ItemAction>) {
        for quality in self.map.values() {
            for (_, item_action_links) in &quality.usages {
                for item_action_link in item_action_links {
                    item_action_link.finalize(item_actions, "quality");
                }
            }
        }
    }
}

impl InfoMap<Recipe> {
    pub(super) fn link_recipes(
        &self,
        qualities: &InfoMap<Quality>,
        requirements: &InfoMap<Requirement>,
        common_item_infos: &InfoMap<CommonItemInfo>,
    ) {
        for recipe in self.map.values() {
            for required_quality in &recipe.qualities.0 {
                required_quality.quality.finalize(qualities, "recipe");
            }

            for alternatives in &recipe.components {
                for alternative in alternatives {
                    match alternative {
                        Alternative::Item { item, .. } => {
                            item.finalize(common_item_infos, "recipe item alternative");
                        }
                        Alternative::Requirement { requirement, .. } => {
                            requirement.finalize(requirements, "recipe requirement alternative");
                        }
                    }
                }
            }

            for using in &recipe.using {
                using.requirement.finalize(requirements, "recipe");
            }

            if let RecipeResult::Item(item_info_link) = &recipe.result {
                item_info_link.finalize(common_item_infos, "recipe");
            }
        }
    }
}

impl InfoMap<Requirement> {
    pub(super) fn link_requirements(
        &self,
        qualities: &InfoMap<Quality>,
        common_item_infos: &InfoMap<CommonItemInfo>,
    ) {
        for requirement in self.map.values() {
            for required_quality in &requirement.qualities.0 {
                required_quality
                    .quality
                    .finalize(qualities, "required quality for requirement");
            }

            for alternatives in &requirement.components {
                for alternative in alternatives {
                    match alternative {
                        Alternative::Item { item, .. } => {
                            item.finalize(common_item_infos, "requirement item alternative");
                        }
                        Alternative::Requirement { requirement, .. } => {
                            requirement.finalize(self, "nested requirement alternative");
                        }
                    }
                }
            }
        }
    }
}

impl InfoMap<TerrainInfo> {
    pub(super) fn fix_and_link_terrain(
        &mut self,
        furniture: &InfoMap<FurnitureInfo>,
        common_item_infos: &InfoMap<CommonItemInfo>,
        item_groups: &InfoMap<ItemGroup>,
    ) {
        if self.map.remove(&InfoId::new("t_null")).is_some() {
            warn!("The terrain t_null was not expected to be present");
        }

        for terrain in self.map.values() {
            terrain.open.finalize(self, "open terrain");
            terrain.close.finalize(self, "closed terrain");
            if let Some(bash) = &terrain.bash {
                link_bash(
                    bash,
                    self,
                    furniture,
                    common_item_infos,
                    item_groups,
                    "terrain",
                );
            }
        }
    }
}

pub(super) fn link_bash(
    bash: &Bash,
    terrain_info: &InfoMap<TerrainInfo>,
    furniture_info: &InfoMap<FurnitureInfo>,
    common_item_infos: &InfoMap<CommonItemInfo>,
    item_groups: &InfoMap<ItemGroup>,
    name: &str,
) {
    bash.terrain
        .finalize(terrain_info, format!("terrain for bashed {name}"));
    bash.furniture
        .finalize(furniture_info, format!("furniture for bashed {name}"));
    if let Some(bash_items) = &bash.items {
        match bash_items {
            BashItems::Explicit(explicit_bash_items) => {
                for bash_item in explicit_bash_items {
                    match bash_item {
                        BashItem::Single(item_occurrence) => item_occurrence
                            .item
                            .finalize(common_item_infos, format!("items for bashed {name}")),
                        BashItem::Group { group } => group.finalize(
                            item_groups,
                            format!("explicit item group for bashed {name}"),
                        ),
                    }
                }
            }
            BashItems::Collection(item_group) => {
                item_group.finalize(item_groups, format!("item collection for bashed {name}"));
            }
        }
    }
}

impl InfoMap<VehiclePartInfo> {
    pub(crate) fn add_wiring(&mut self) {
        let id = InfoId::new("wiring");
        self.map.entry(id.clone()).or_insert_with(|| {
            Arc::new(VehiclePartInfo {
                id,
                name: Some(CddaItemName::Simple(Arc::from("Wiring")).into()),
                item: RequiredLinkedLater::new(InfoId::new("wire")),
                looks_like: None,
                flags: None.into(),
            })
        });
    }

    pub(super) fn add_vehicle_part_migrations<'a>(
        &mut self,
        vehicle_part_migrations: impl Iterator<Item = &'a Arc<VehiclePartMigration>>,
    ) {
        // TODO Make this recursive
        for migration in vehicle_part_migrations {
            if let Ok(new) = self.get(&migration.from).cloned() {
                self.map.insert(migration.from.clone(), new);
            }
        }
    }

    pub(super) fn link_items(&self, common_item_infos: &InfoMap<CommonItemInfo>) {
        for part_info in self.map.values() {
            part_info
                .item
                .finalize(common_item_infos, "vehicle part item");
        }
    }
}

impl<T: DeserializeOwned + 'static> Default for InfoMap<T> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            aliases: HashMap::default(),
        }
    }
}

impl<T: fmt::Debug + DeserializeOwned + 'static> LinkProvider<T> for InfoMap<T> {
    fn get_option(&self, info_id: &InfoId<T>) -> Option<&Arc<T>> {
        self.get(info_id).ok()
    }
}

pub(super) struct ItemInfoMapLoader<'a> {
    pub(super) enriched_json_infos: &'a mut HashMap<TypeId, HashMap<UntypedInfoId, Enriched>>,
    pub(super) item_migrations: InfoMap<ItemMigration>,
    pub(super) common_item_infos: &'a mut InfoMap<CommonItemInfo>,
}

impl ItemInfoMapLoader<'_> {
    pub(super) fn item_extract<T>(&mut self, type_id: TypeId) -> InfoMap<T>
    where
        T: fmt::Debug + DeserializeOwned + ItemWithCommonInfo + 'static,
    {
        let mut items = InfoMap::<T>::new(self.enriched_json_infos, type_id);

        // TODO Make this recursive
        for migration in self.item_migrations.values() {
            if let Ok(new) = items
                .get(&migration.replace.untyped().clone().into())
                .cloned()
            {
                items
                    .aliases
                    .insert(migration.id.untyped().clone().into(), new);
            }
        }

        for (id, item_info) in &mut items.map {
            let previous = self
                .common_item_infos
                .map
                .insert(id.untyped().clone().into(), item_info.common());
            if let Some(previous) = previous {
                warn!(
                    "Item {id:?} replaced the existing common item {:?}",
                    previous.id
                );
            }
        }

        for (alias, item_info) in &mut items.aliases {
            let previous = self
                .common_item_infos
                .aliases
                .insert(alias.untyped().clone().into(), item_info.common());
            if let Some(previous) = previous {
                warn!(
                    "Item alias {alias:?} replaced the existing common item {:?}",
                    previous.id
                );
            }
        }

        //trace!(
        //    "{}x {}, and {}x common items",
        //    items.map.len(),
        //    std::any::type_name::<T>(),
        //    self.common_item_infos.map.len()
        //);

        items
    }
}
