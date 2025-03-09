use crate::{HashMap, ObjectId};
use serde::{Deserialize, Deserializer};
use std::cell::RefCell;
use std::fmt;
use std::sync::{Arc, OnceLock, Weak};

#[derive(Debug, Deserialize)]
#[serde(from = "Option<LinkedLater<T>>")]
pub struct OptionalLinkedLater<T> {
    pub option: Option<LinkedLater<T>>,
}

impl<T: fmt::Debug> OptionalLinkedLater<T> {
    pub fn get(&self) -> Option<Arc<T>> {
        self.option.as_ref().and_then(LinkedLater::get)
    }

    /// All finalize methods combined may only be called once
    pub fn finalize_arc(&self, map: &HashMap<ObjectId, Arc<T>>, err_description: &str) {
        if let Some(linked_later) = &self.option {
            linked_later.finalize_arc(map, err_description);
        }
    }

    /// All finalize methods combined may only be called once
    pub fn finalize_refcell_arc(
        &self,
        map: &HashMap<ObjectId, RefCell<Arc<T>>>,
        err_description: &str,
    ) {
        if let Some(linked_later) = &self.option {
            linked_later.finalize_refcell_arc(map, err_description);
        }
    }
}

impl<T> From<Option<LinkedLater<T>>> for OptionalLinkedLater<T> {
    fn from(option: Option<LinkedLater<T>>) -> Self {
        Self { option }
    }
}

#[derive(Debug)]
pub struct LinkedLater<T> {
    object_id: ObjectId,
    lock: OnceLock<Weak<T>>,
}

impl<T: fmt::Debug> LinkedLater<T> {
    pub fn get(&self) -> Option<Arc<T>> {
        self.lock.get().map(|lock| {
            lock.upgrade()
                .expect("Referenced value should be available")
        })
    }

    /// All finalize methods combined may only be called once
    pub fn finalize_arc(&self, map: &HashMap<ObjectId, Arc<T>>, err_description: &str) {
        self.finalize_found(map.get(&self.object_id).map(Arc::downgrade), err_description)
    }

    /// All finalize methods combined may only be called once
    pub fn finalize_refcell_arc(
        &self,
        map: &HashMap<ObjectId, RefCell<Arc<T>>>,
        err_description: &str,
    ) {
        self.finalize_found(
            map.get(&self.object_id)
                .map(RefCell::borrow)
                .map(|x| Arc::downgrade(&*x)),
            err_description,
        )
    }

    fn finalize_found(&self, found: Option<Weak<T>>, err_description: &str) {
        if let Some(weak) = found {
            self.lock
                .set(weak)
                .expect("{self:?}.finalize() should still be allowed");
        } else {
            eprintln!("Could not find {err_description}: {self:?}")
        }
    }
}

impl<'de, T> Deserialize<'de> for LinkedLater<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            object_id: Deserialize::deserialize(deserializer)?,
            lock: OnceLock::new(),
        })
    }
}
