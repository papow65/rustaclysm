use crate::prelude::*;
use bevy::{ecs::system::Resource, utils::HashMap};
use glob::glob;

#[derive(Resource)]
pub(crate) struct ItemInfos {
    infos: HashMap<ObjectName, ItemInfo>,
}

impl ItemInfos {
    pub(crate) fn new() -> Self {
        Self {
            infos: HashMap::default(),
        }
        .load_all()
    }

    pub(crate) fn get<'a>(&'a self, name: &'a ObjectName) -> Option<&'a ItemInfo> {
        self.infos.get(name)
    }

    fn load_all(mut self) -> Self {
        let mut cdda_infos = HashMap::<ObjectName, CddaItemInfo>::new();
        let items_path = Paths::data_path().join("json").join("items");
        let pattern = items_path.join("**").join("*.json");
        let pattern = pattern.to_str().unwrap();
        for json_path in glob(pattern).expect("Failed to read glob pattern") {
            let json_path = json_path.expect("problem with json path for item infos");
            let item_info_list = CddaItemInfoList::try_from(json_path);
            let item_info_list = item_info_list.expect("Failed loading item infos");

            for item_info in item_info_list.0 {
                let id = item_info
                    .id
                    .clone()
                    .or_else(|| item_info.abstract_.clone())
                    .unwrap();
                cdda_infos.insert(id, item_info);
            }
        }

        for cdda_info in cdda_infos.values() {
            if let Some(id) = &cdda_info.id {
                let mut stack = vec![cdda_info];
                while let Some(parent_id) = &stack.last().unwrap().copy_from {
                    stack.push(cdda_infos.get(parent_id).unwrap());
                }
                self.infos.insert(id.clone(), ItemInfo::new(&stack));
            }
        }

        assert!(!self.infos.is_empty(), "No item infos found");
        self
    }
}
