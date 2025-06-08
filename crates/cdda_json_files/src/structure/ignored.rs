use crate::Recipe;
use bevy_log::error;
use bevy_platform::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use std::any::{TypeId, type_name};
use std::sync::{Arc, Mutex, OnceLock};
use std::{collections::BTreeSet, marker::PhantomData};

#[derive(Debug)]
pub struct Ignored<T> {
    _fields: HashMap<Arc<str>, JsonValue>,
    _phantom: PhantomData<T>,
}

impl<'de, T: 'static> Deserialize<'de> for Ignored<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        static ALL_UNUSED_FIELDS: OnceLock<Mutex<HashMap<TypeId, BTreeSet<String>>>> =
            OnceLock::new();
        let all_unused_fields = ALL_UNUSED_FIELDS.get_or_init(|| Mutex::new(HashMap::default()));

        let unused_fields: HashMap<Arc<str>, JsonValue> = Deserialize::deserialize(deserializer)?;
        let new_ignored_fields = unused_fields
            .iter()
            .filter(|(key, _)| {
                !key.starts_with("//")
                    && (TypeId::of::<T>() != TypeId::of::<Recipe>()
                        || !["result", "category", "description"].contains(&&***key))
            })
            .map(|(key, value)| format!("{key} ({})", variant_name(value)))
            .filter(|field| {
                !all_unused_fields
                    .lock()
                    .expect("The mutex should not be poisoned")
                    .entry(TypeId::of::<T>())
                    .or_default()
                    .contains(field)
            })
            .collect::<Vec<_>>();
        if !new_ignored_fields.is_empty() {
            let mut all_unused_fields = all_unused_fields
                .lock()
                .expect("The mutex should not be poisoned");
            let all_unused_fields = all_unused_fields
                .get_mut(&TypeId::of::<T>())
                .expect("Type should be found");
            all_unused_fields.extend(new_ignored_fields);
            error!(
                "Ignored fields for {}: {all_unused_fields:?}",
                type_name::<T>(),
            );
        }

        Ok(Self {
            _fields: unused_fields,
            _phantom: PhantomData,
        })
    }
}

impl<T> Default for Ignored<T> {
    fn default() -> Self {
        Self {
            _fields: HashMap::default(),
            _phantom: PhantomData,
        }
    }
}

const fn variant_name(value: &JsonValue) -> &'static str {
    match value {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "bool",
        JsonValue::Number(_) => "number",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(_) => "object",
    }
}
