use crate::{Error, InfoId, InfoIdDescription, TerrainInfo};
use bevy_log::{error, warn};
use serde::Deserialize;
use std::fmt;
use std::sync::{Arc, OnceLock, Weak};

pub trait LinkProvider<T> {
    fn get_option(&self, info_id: &InfoId<T>) -> Option<&Arc<T>>;
}

pub trait Link<T> {
    /// Panics when called a second time
    /// # Errors
    /// When the link could not be established
    fn connect(&self, provider: &impl LinkProvider<T>) -> Result<(), Error>;

    /// Calls [`Self::connect`], and logs the error
    fn finalize(&self, provider: &impl LinkProvider<T>, err_description: impl AsRef<str>) {
        if let Err(error) = self.connect(provider) {
            let err_description = err_description.as_ref();
            error!("Linking {err_description} failed: {error:?}");
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "Option<InfoId<T>>")]
pub struct OptionalLinkedLater<T: fmt::Debug> {
    option: Option<LinkedLater<T>>,
}

impl<T: fmt::Debug> OptionalLinkedLater<T> {
    #[must_use]
    pub const fn new_final_none() -> Self {
        Self { option: None }
    }

    pub fn get(&self) -> Option<Arc<T>> {
        self.option.as_ref().and_then(LinkedLater::get)
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

impl<T: fmt::Debug> Link<T> for OptionalLinkedLater<T> {
    fn connect(&self, provider: &impl LinkProvider<T>) -> Result<(), Error> {
        self.option.as_ref().map_or(Ok(()), |linked_later| {
            linked_later.finalize(
                provider
                    .get_option(&linked_later.info_id)
                    .map(Arc::downgrade),
            )
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "InfoId<T>")]
pub struct RequiredLinkedLater<T: fmt::Debug> {
    required: LinkedLater<T>,
}

impl<T: fmt::Debug + 'static> RequiredLinkedLater<T> {
    #[must_use]
    pub fn new(info_id: InfoId<T>) -> Self {
        Self {
            required: LinkedLater::new(info_id),
        }
    }

    #[must_use]
    pub fn new_final(info_id: InfoId<T>, value: &Arc<T>) -> Self {
        Self {
            required: LinkedLater::new_final(info_id, value),
        }
    }

    /// # Errors
    /// When the link is unavailable
    pub fn get(&self) -> Result<Arc<T>, Error> {
        self.required.get().ok_or_else(|| Error::LinkUnavailable {
            _id: InfoIdDescription::from(self.required.info_id.clone()),
        })
    }

    /// Logs the error that [`Self.get`] would give and converts the result to an option
    pub fn get_option(&self, called_from: impl AsRef<str>) -> Option<Arc<T>> {
        self.get()
            .inspect_err(|error| warn!("{} caused {error:#?}", called_from.as_ref()))
            .ok()
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

impl<T: fmt::Debug> Link<T> for RequiredLinkedLater<T> {
    fn connect(&self, provider: &impl LinkProvider<T>) -> Result<(), Error> {
        self.required.finalize(
            provider
                .get_option(&self.required.info_id)
                .map(Arc::downgrade),
        )
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
            .unwrap_or_else(|| panic!("{:?} should have been finalized before usage", self.info_id))
            .as_ref()
            .map(|weak| {
                weak.upgrade()
                    .expect("Referenced value should be available")
            })
    }

    /// This may only be called once.
    fn finalize(&self, found: Option<Weak<T>>) -> Result<(), Error> {
        // This should not happen, even with malformed data. So we use 'expect()' instead of returning an error.
        self.lock.set(found).expect("{self:?} is already finalized");

        if self.lock.get().is_none() {
            return Err(Error::UnknownInfoId {
                _id: self.info_id.clone().into(),
            });
        }

        Ok(())
    }
}
