use crate::{HashMap, ObjectId};
use serde::Deserialize;
use std::cell::RefCell;
use std::fmt;
use std::sync::{Arc, OnceLock, Weak};

#[derive(Debug, Deserialize)]
#[serde(from = "Option<ObjectId>")]
pub struct OptionalLinkedLater<T: fmt::Debug> {
    option: Option<LinkedLater<T>>,
}

impl<T: fmt::Debug> OptionalLinkedLater<T> {
    pub fn get(&self) -> Option<Arc<T>> {
        self.option.as_ref().and_then(LinkedLater::get)
    }

    /// All finalize methods combined may only be called once
    pub fn finalize_arc(&self, map: &HashMap<ObjectId, Arc<T>>, err_description: &str) {
        self.option.as_ref().map(|linked_later| {
            linked_later.finalize(
                map.get(&linked_later.object_id).map(Arc::downgrade),
                err_description,
            )
        });
    }

    /// All finalize methods combined may only be called once
    pub fn finalize_refcell_arc(
        &self,
        map: &HashMap<ObjectId, RefCell<Arc<T>>>,
        err_description: &str,
    ) {
        self.option.as_ref().map(|linked_later| {
            linked_later.finalize(
                map.get(&linked_later.object_id)
                    .map(|refcell| Arc::downgrade(&*refcell.borrow())),
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

#[derive(Debug)]
struct LinkedLater<T: fmt::Debug> {
    object_id: ObjectId,
    lock: OnceLock<Weak<T>>,
}

impl<T: fmt::Debug> LinkedLater<T> {
    fn get(&self) -> Option<Arc<T>> {
        self.lock.get().map(|lock| {
            lock.upgrade()
                .expect("Referenced value should be available")
        })
    }

    fn finalize(&self, found: Option<Weak<T>>, err_description: &str) {
        if let Some(weak) = found {
            self.lock.set(weak).expect("{self:?} is already finalized");
        } else {
            eprintln!("Could not find {err_description}: {self:?}");
        }
    }
}
