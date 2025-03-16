use crate::{Error, HashMap, InfoId, InfoIdDescription, TerrainInfo};
use bevy_log::{error, warn};
use serde::Deserialize;
use std::fmt;
use std::sync::{Arc, OnceLock, Weak};

#[derive(Debug, Deserialize)]
#[serde(from = "Option<InfoId<T>>")]
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
    pub fn finalize<'a>(
        &self,
        map: &HashMap<InfoId<T>, Arc<T>>,
        err_description: impl Into<&'a str>,
    ) {
        self.option.as_ref().map(|linked_later| {
            linked_later.finalize(
                map.get(&linked_later.object_id).map(Arc::downgrade),
                err_description.into(),
            )
        });
    }
}

impl<T: fmt::Debug> From<Option<InfoId<T>>> for OptionalLinkedLater<T> {
    fn from(object_id: Option<InfoId<T>>) -> Self {
        Self {
            option: object_id.map(LinkedLater::new),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "Vec<(InfoId<T>, U)>")]
pub struct VecLinkedLater<T: fmt::Debug, U: Clone + fmt::Debug> {
    vec: Vec<(LinkedLater<T>, U)>,
}

impl<T: fmt::Debug, U: Clone + fmt::Debug> VecLinkedLater<T, U> {
    pub fn new_final_empty() -> Self {
        Self { vec: Vec::new() }
    }

    /// May only be called once
    pub fn finalize<'a>(
        &self,
        map: &HashMap<InfoId<T>, Arc<T>>,
        err_description: impl Into<&'a str>,
    ) {
        let err_description = err_description.into();
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

impl<T: fmt::Debug, U: Clone + fmt::Debug> From<Vec<(InfoId<T>, U)>> for VecLinkedLater<T, U> {
    fn from(vec: Vec<(InfoId<T>, U)>) -> Self {
        Self {
            vec: vec
                .into_iter()
                .map(|(object_id, assoc)| (LinkedLater::new(object_id), assoc))
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "InfoId<T>")]
pub struct RequiredLinkedLater<T: fmt::Debug> {
    required: LinkedLater<T>,
}

impl<T: fmt::Debug + 'static> RequiredLinkedLater<T> {
    pub fn new_final(object_id: InfoId<T>, value: &Arc<T>) -> Self {
        Self {
            required: LinkedLater::new_final(object_id, value),
        }
    }

    pub fn get(&self) -> Result<Arc<T>, Error> {
        self.required.get().ok_or_else(|| Error::LinkUnavailable {
            _id: InfoIdDescription::from(self.required.object_id.clone()),
        })
    }

    /// Logs the error that [`Self.get`] would give and converts the result to an option
    pub fn get_option<'a>(&'a self, called_from: impl AsRef<str>) -> Option<Arc<T>> {
        self.get()
            .inspect_err(|error| warn!("{} caused {error:#?}", called_from.as_ref()))
            .ok()
    }

    /// May only be called once
    pub fn finalize<'a>(
        &self,
        map: &HashMap<InfoId<T>, Arc<T>>,
        err_description: impl Into<&'a str>,
    ) {
        self.required.finalize(
            map.get(&self.required.object_id).map(Arc::downgrade),
            err_description.into(),
        );
    }
}

impl RequiredLinkedLater<TerrainInfo> {
    pub(crate) fn is_significant(&self) -> bool {
        ![
            InfoId::new("t_open_air"),
            InfoId::new("t_soil"),
            InfoId::new("t_rock"),
        ]
        .contains(&self.required.object_id)
    }
}

impl<T: fmt::Debug> From<InfoId<T>> for RequiredLinkedLater<T> {
    fn from(object_id: InfoId<T>) -> Self {
        Self {
            required: LinkedLater::new(object_id),
        }
    }
}

#[derive(Debug)]
struct LinkedLater<T: fmt::Debug> {
    object_id: InfoId<T>,
    lock: OnceLock<Option<Weak<T>>>,
}

impl<T: fmt::Debug> LinkedLater<T> {
    fn new(object_id: InfoId<T>) -> Self {
        Self {
            object_id,
            lock: OnceLock::default(),
        }
    }

    fn new_final(object_id: InfoId<T>, value: &Arc<T>) -> Self {
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
            error!("Could not find {err_description}: {:?}", &self.object_id);
        }
        self.lock.set(found).expect("{self:?} is already finalized");
    }
}
