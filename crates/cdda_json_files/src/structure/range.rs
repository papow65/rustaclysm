use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(from = "CddaRange<T, N>")]
pub struct Range<T, N: Copy + Clone> {
    pub obj: T,
    pub min: N,
    pub max: N,
}

impl<T, N: Copy + Clone> From<CddaRange<T, N>> for Range<T, N> {
    fn from(range: CddaRange<T, N>) -> Self {
        match range {
            CddaRange::Exact(obj, exact) => Self {
                obj,
                min: exact,
                max: exact,
            },
            CddaRange::Between(obj, min, max) => Self { obj, min, max },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CddaRange<T, N: Copy + Clone> {
    Exact(T, N),
    Between(T, N, N),
}
