use crate::{Error, HashMap, ObjectId, TerrainInfo};
use serde::Deserialize;
use std::sync::{Arc, OnceLock, Weak};
use std::{any::type_name, fmt};

#[derive(Debug, Deserialize)]
#[serde(from = "Option<ObjectId>")]
pub struct OptionalLinkedLater<T: fmt::Debug> {
    option: Option<LinkedLater<T>>,
}

impl<T: fmt::Debug> OptionalLinkedLater<T> {
    pub fn new_final_none() -> Self {
        Self { option: None }
    }

    pub fn get(&self) -> Option<Arc<T>> {
        self.option.as_ref().and_then(LinkedLater::get)
    }

    /// May only be called once
    pub fn finalize(&self, map: &HashMap<ObjectId, Arc<T>>, err_description: &str) {
        self.option.as_ref().map(|linked_later| {
            linked_later.finalize(
                map.get(&linked_later.object_id).map(Arc::downgrade),
                err_description,
            )
        });
    }
}

impl<T: fmt::Debug> From<Option<ObjectId>> for OptionalLinkedLater<T> {
    fn from(object_id: Option<ObjectId>) -> Self {
        Self {
            option: object_id.map(|object_id| LinkedLater {
                object_id,
                lock: OnceLock::default(),
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "Vec<(ObjectId, U)>")]
pub struct VecLinkedLater<T: fmt::Debug, U: Clone + fmt::Debug> {
    vec: Vec<(LinkedLater<T>, U)>,
}

impl<T: fmt::Debug, U: Clone + fmt::Debug> VecLinkedLater<T, U> {
    pub fn new_final_empty() -> Self {
        Self { vec: Vec::new() }
    }

    /// May only be called once
    pub fn finalize(&self, map: &HashMap<ObjectId, Arc<T>>, err_description: &str) {
        for (linked_later, _assoc) in &self.vec {
            linked_later.finalize(
                map.get(&linked_later.object_id).map(Arc::downgrade),
                err_description,
            );
        }
    }

    pub fn get_all(&self) -> Vec<(Arc<T>, U)> {
        self.vec
            .iter()
            .filter_map(|(linked_later, assoc)| {
                linked_later.get().map(|item| (item.clone(), assoc.clone()))
            })
            .collect()
    }
}

impl<T: fmt::Debug, U: Clone + fmt::Debug> Default for VecLinkedLater<T, U> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
        }
    }
}

impl<T: fmt::Debug, U: Clone + fmt::Debug> From<Vec<(ObjectId, U)>> for VecLinkedLater<T, U> {
    fn from(vec: Vec<(ObjectId, U)>) -> Self {
        Self {
            vec: vec
                .into_iter()
                .map(|(object_id, assoc)| {
                    (
                        LinkedLater {
                            object_id,
                            lock: OnceLock::default(),
                        },
                        assoc,
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "ObjectId")]
pub struct RequiredLinkedLater<T: fmt::Debug> {
    required: LinkedLater<T>,
}

impl<T: fmt::Debug + 'static> RequiredLinkedLater<T> {
    pub fn new_final(object_id: ObjectId, value: &Arc<T>) -> Self {
        Self {
            required: LinkedLater::new_final(object_id, value),
        }
    }

    pub fn get(&self) -> Result<Arc<T>, Error> {
        self.required.get().ok_or_else(|| Error::LinkUnavailable {
            _id: self.required.object_id.clone(),
            _type: type_name::<T>(),
        })
    }

    pub fn get_or<F: FnOnce(&Error)>(&self, handle_error: F) -> Option<Arc<T>> {
        self.get().inspect_err(handle_error).ok()
    }

    /// May only be called once
    pub fn finalize(&self, map: &HashMap<ObjectId, Arc<T>>, err_description: &str) {
        self.required.finalize(
            map.get(&self.required.object_id).map(Arc::downgrade),
            err_description,
        );
    }
}

impl RequiredLinkedLater<TerrainInfo> {
    pub(crate) fn is_significant(&self) -> bool {
        ![
            ObjectId::new("t_open_air"),
            ObjectId::new("t_soil"),
            ObjectId::new("t_rock"),
        ]
        .contains(&self.required.object_id)
    }
}

impl<T: fmt::Debug> From<ObjectId> for RequiredLinkedLater<T> {
    fn from(object_id: ObjectId) -> Self {
        Self {
            required: LinkedLater {
                object_id,
                lock: OnceLock::default(),
            },
        }
    }
}

#[derive(Debug)]
struct LinkedLater<T: fmt::Debug> {
    object_id: ObjectId,
    lock: OnceLock<Option<Weak<T>>>,
}

impl<T: fmt::Debug> LinkedLater<T> {
    fn new_final(object_id: ObjectId, value: &Arc<T>) -> Self {
        let lock = OnceLock::new();
        lock.set(Some(Arc::downgrade(value)))
            .expect("This lock should be unused");

        Self { object_id, lock }
    }

    fn get(&self) -> Option<Arc<T>> {
        self.lock
            .get()
            .expect("Should be finalized")
            .as_ref()
            .map(|weak| {
                weak.upgrade()
                    .expect("Referenced value should be available")
            })
    }

    fn finalize(&self, found: Option<Weak<T>>, err_description: &str) {
        if found.is_none() {
            eprintln!("Could not find {err_description}: {:?}", &self.object_id);
        }
        self.lock.set(found).expect("{self:?} is already finalized");
    }
}
