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
    pub fn finalize(&self, map: &HashMap<InfoId<T>, Arc<T>>, err_description: impl AsRef<str>) {
        self.option.as_ref().map(|linked_later| {
            linked_later.finalize(
                map.get(&linked_later.info_id).map(Arc::downgrade),
                err_description.as_ref(),
            )
        });
    }
}

impl<T: fmt::Debug> From<Option<InfoId<T>>> for OptionalLinkedLater<T> {
    fn from(mut info_id: Option<InfoId<T>>) -> Self {
        if info_id == Some(InfoId::new("t_null")) {
            info_id = None;
        }

        Self {
            option: info_id.map(LinkedLater::new),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "InfoId<T>")]
pub struct RequiredLinkedLater<T: fmt::Debug> {
    required: LinkedLater<T>,
}

impl<T: fmt::Debug + 'static> RequiredLinkedLater<T> {
    pub fn new_final(info_id: InfoId<T>, value: &Arc<T>) -> Self {
        Self {
            required: LinkedLater::new_final(info_id, value),
        }
    }

    pub fn get(&self) -> Result<Arc<T>, Error> {
        self.required.get().ok_or_else(|| Error::LinkUnavailable {
            _id: InfoIdDescription::from(self.required.info_id.clone()),
        })
    }

    /// Logs the error that [`Self.get`] would give and converts the result to an option
    pub fn get_option<'a>(&'a self, called_from: impl AsRef<str>) -> Option<Arc<T>> {
        self.get()
            .inspect_err(|error| warn!("{} caused {error:#?}", called_from.as_ref()))
            .ok()
    }

    /// May only be called once
    pub fn finalize<'a>(&self, map: &HashMap<InfoId<T>, Arc<T>>, err_description: impl AsRef<str>) {
        self.required.finalize(
            map.get(&self.required.info_id).map(Arc::downgrade),
            err_description.as_ref(),
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
        .contains(&self.required.info_id)
    }
}

impl<T: fmt::Debug> From<InfoId<T>> for RequiredLinkedLater<T> {
    fn from(info_id: InfoId<T>) -> Self {
        Self {
            required: LinkedLater::new(info_id),
        }
    }
}

#[derive(Debug)]
struct LinkedLater<T: fmt::Debug> {
    info_id: InfoId<T>,
    lock: OnceLock<Option<Weak<T>>>,
}

impl<T: fmt::Debug> LinkedLater<T> {
    fn new(info_id: InfoId<T>) -> Self {
        Self {
            info_id,
            lock: OnceLock::default(),
        }
    }

    fn new_final(info_id: InfoId<T>, value: &Arc<T>) -> Self {
        let lock = OnceLock::new();
        lock.set(Some(Arc::downgrade(value)))
            .expect("This lock should be unused");

        Self { info_id, lock }
    }

    fn get(&self) -> Option<Arc<T>> {
        self.lock
            .get()
            .expect("This link should have been finalized before usage")
            .as_ref()
            .map(|weak| {
                weak.upgrade()
                    .expect("Referenced value should be available")
            })
    }

    fn finalize(&self, found: Option<Weak<T>>, err_description: &str) {
        if found.is_none() {
            error!("Could not find {err_description}: {:?}", &self.info_id);
        }
        self.lock.set(found).expect("{self:?} is already finalized");
    }
}
