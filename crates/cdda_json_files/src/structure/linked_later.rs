use crate::ObjectId;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::sync::{Arc, OnceLock, Weak};

#[derive(Debug)]
pub struct LinkedLater<T> {
    object_id: ObjectId,
    lock: OnceLock<Weak<T>>,
}

impl<T: fmt::Debug> LinkedLater<T> {
    /// The caller may only call this before [`Self.finalize`]
    pub fn initial(&self) -> &ObjectId {
        if self.lock.get().is_none() {
            &self.object_id
        } else {
            panic!("{self:?}.initial() is no longer allowe;");
        }
    }

    pub fn get(&self) -> Option<Arc<T>> {
        self.lock.get().map(|lock| {
            lock.upgrade()
                .expect("Referenced value should be available")
        })
    }

    /// The caller may only call this once
    pub fn finalize(&self, value: &Arc<T>) {
        self.lock
            .set(Arc::downgrade(value))
            .expect("{self:?}.finalize() is no longer allowed");
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
