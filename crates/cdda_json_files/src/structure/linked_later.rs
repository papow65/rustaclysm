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
